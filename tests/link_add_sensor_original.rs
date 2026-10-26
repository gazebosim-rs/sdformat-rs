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
// Observable values from DOMLink.AddSensor.
use sdformat::{Link, NoResources, Sensor, load_str};

fn link_and_sensor() -> (Link, Sensor) {
    let mut link = load_str(
        "<sdf version='1.12'><model name='parent'><link name='link'><sensor name='sensor1' type='imu'><imu/></sensor></link></model></sdf>",
        "link-add-sensor.sdf",
        &NoResources,
    )
    .unwrap()
    .models
    .remove(0)
    .links
    .remove(0);
    let sensor = link.sensors.remove(0);
    (link, sensor)
}

#[test]
fn original_dom_link_add_sensor() {
    let (mut link, sensor) = link_and_sensor();
    assert_eq!(link.sensors.len(), 0);

    assert!(link.add_sensor_unique(sensor.clone()));
    assert_eq!(link.sensors.len(), 1);
    assert!(!link.add_sensor_unique(sensor.clone()));
    assert_eq!(link.sensors.len(), 1);

    link.clear_sensors();
    assert_eq!(link.sensors.len(), 0);
    assert!(link.add_sensor_unique(sensor));
    assert_eq!(link.sensors.len(), 1);
    let sensor_from_link = link.sensors.first().expect("SensorByIndex(0)");
    assert_eq!(sensor_from_link.name, "sensor1");
}
