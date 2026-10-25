/*
 * Copyright (C) 2018 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Pinned src/Sensor_TEST.cc DOMSensor.Plugins, lines 584-605.
use sdformat::{NoResources, Plugin, Sensor, load_str};

fn sensor(xml: &str) -> Sensor {
    let mut parsed = load_str(xml, "sensor-plugins.sdf", &NoResources).unwrap();
    parsed.models.remove(0).links.remove(0).sensors.remove(0)
}

#[test]
fn original_dom_sensor_plugins() {
    let mut sensor = sensor(
        "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='magnetometer'/></link></model></sdf>",
    );
    assert!(sensor.plugins.is_empty());

    let mut plugin = Plugin {
        name: "name1".into(),
        filename: "filename1".into(),
        xml: String::new(),
    };
    sensor.plugins.push(plugin.clone());
    assert_eq!(sensor.plugins.len(), 1);

    plugin.name = "name2".into();
    sensor.plugins.push(plugin);
    assert_eq!(sensor.plugins.len(), 2);
    assert_eq!(sensor.plugins[0].name, "name1");
    assert_eq!(sensor.plugins[1].name, "name2");

    sensor.plugins.clear();
    assert!(sensor.plugins.is_empty());
}

#[test]
fn parsed_sensor_plugins_keep_order_and_original_xml() {
    let first = "<plugin name='name1' filename='filename1'/>";
    let second = "<plugin name='name2' filename='filename1'/>";
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='magnetometer'>{first}{second}</sensor></link></model></sdf>"
    );
    let sensor = sensor(&xml);
    assert_eq!(sensor.plugins.len(), 2);
    assert_eq!(sensor.plugins[0].name, "name1");
    assert_eq!(sensor.plugins[0].filename, "filename1");
    assert_eq!(sensor.plugins[0].xml, first);
    assert_eq!(sensor.plugins[1].name, "name2");
    assert_eq!(sensor.plugins[1].filename, "filename1");
    assert_eq!(sensor.plugins[1].xml, second);
}
