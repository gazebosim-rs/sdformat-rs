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
// Observable values from DOMJoint.MutableByIndex and MutableByName.
use sdformat::{Joint, NoResources, Sensor, load_str};

fn parsed_joint() -> Joint {
    let xml = "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='fixed'><parent>a</parent><child>b</child></joint></model></sdf>";
    load_str(xml, "joint-mutable.sdf", &NoResources)
        .unwrap()
        .models[0]
        .joints[0]
        .clone()
}

fn parsed_sensor() -> Sensor {
    let xml = "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='sensor1' type='imu'/></joint></model></sdf>";
    load_str(xml, "joint-sensor.sdf", &NoResources)
        .unwrap()
        .models[0]
        .joints[0]
        .sensors[0]
        .clone()
}

#[test]
fn original_dom_joint_mutable_by_index() {
    let mut joint = parsed_joint();
    assert!(joint.sensors.is_empty());
    joint.sensors.push(parsed_sensor());
    assert_eq!(joint.sensors.len(), 1);
    let sensor = joint.sensors.get_mut(0).expect("SensorByIndex(0)");
    assert_eq!(sensor.name, "sensor1");
    sensor.name = "sensor2".into();
    assert_eq!(joint.sensors.first().unwrap().name, "sensor2");
}

#[test]
fn original_dom_joint_mutable_by_name() {
    let mut joint = parsed_joint();
    assert!(joint.sensors.is_empty());
    joint.sensors.push(parsed_sensor());
    assert_eq!(joint.sensors.len(), 1);
    let sensor = joint
        .sensors
        .iter_mut()
        .find(|sensor| sensor.name == "sensor1")
        .expect("SensorByName(sensor1)");
    assert_eq!(sensor.name, "sensor1");
    sensor.name = "sensor2".into();
    assert_eq!(
        joint
            .sensors
            .iter()
            .find(|sensor| sensor.name == "sensor2")
            .unwrap()
            .name,
        "sensor2"
    );
}
