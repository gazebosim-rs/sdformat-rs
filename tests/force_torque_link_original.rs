/*
 * Copyright 2018 Open Source Robotics Foundation
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

use sdformat::{ForceTorque, NoResources, SensorConfig, SensorType, load_str};

#[test]
fn native_link_force_torque_sensor_fragment_is_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let tag = "<sensor name=\"force_torque_sensor\" type=\"force_torque\">";
    let start = fixture.find(tag).unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let xml = format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{}</link></model></sdf>",
        &fixture[start..end]
    );
    let sdf = load_str(&xml, "sensors.sdf#force-torque", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "force_torque_sensor");
    assert_eq!(sensor.kind, "force_torque");
    assert_eq!(sensor.effective_type(), Some(SensorType::ForceTorque));
    assert!(matches!(sensor.config, SensorConfig::ForceTorque(_)));
    assert_eq!(sensor.pose.transform.translation, [10., 11., 12.]);
    assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(!sensor.enable_metrics);
    assert_eq!(sensor.force_torque(), Some(&ForceTorque::default()));
    assert!(sensor.configuration_xml.is_empty());
}
