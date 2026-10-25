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
// Pinned src/Sensor_TEST.cc DOMSensor copy/move originals, lines 81-238.
use sdformat::{NoResources, Sensor, SensorType, load_str};

fn sensor(kind: &str) -> Sensor {
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='{kind}'/></link></model></sdf>"
    );
    let mut parsed = load_str(&xml, "sensor-ownership.sdf", &NoResources).unwrap();
    parsed.models.remove(0).links.remove(0).sensors.remove(0)
}

fn magnetometer(rate: f64, relative_to: &str, frame_id: &str) -> Sensor {
    let mut value = sensor("magnetometer");
    value.pose.transform.translation = [1., 2., 3.];
    value.pose.relative_to = relative_to.into();
    value.frame_id = frame_id.into();
    value.update_rate = rate;
    if let sdformat::SensorConfig::Magnetometer(magnetometer) = &mut value.config {
        magnetometer.noise[0].mean = 0.1;
    } else {
        panic!("magnetometer payload");
    }
    value
}

fn original_magnetometer_values(value: &Sensor, relative_to: &str, frame_id: &str) {
    assert_eq!(value.effective_type(), Some(SensorType::Magnetometer));
    assert_eq!(value.pose.transform.translation, [1., 2., 3.]);
    assert_eq!(value.pose.transform.rotation, [0., 0., 0., 1.]);
    assert_eq!(value.pose.relative_to, relative_to);
    assert_eq!(value.frame_id, frame_id);
    assert_eq!(value.magnetometer().unwrap().noise[0].mean, 0.1);
}

#[test]
fn original_dom_sensor_move_constructor() {
    let original = magnetometer(0.123, "a_frame", "new_frame_id");
    let moved = original;
    original_magnetometer_values(&moved, "a_frame", "new_frame_id");
    assert_eq!(moved.update_rate, 0.123);
}

#[test]
fn original_dom_sensor_copy_constructor() {
    let original = magnetometer(0.123, "a_frame", "new_frame_id");
    let copied = original.clone();
    original_magnetometer_values(&original, "a_frame", "new_frame_id");
    original_magnetometer_values(&copied, "a_frame", "new_frame_id");
    assert_eq!(copied.update_rate, 0.123);
}

#[test]
fn original_dom_sensor_move_assignment() {
    let original = magnetometer(0., "a_frame", "new_frame_id");
    let mut assigned = sensor("camera");
    assert_eq!(assigned.effective_type(), Some(SensorType::Camera));
    assigned = original;
    original_magnetometer_values(&assigned, "a_frame", "new_frame_id");
}

#[test]
fn original_dom_sensor_copy_assignment() {
    let original = magnetometer(0., "a_frame", "new_frame_id");
    let mut assigned = sensor("camera");
    assert_eq!(assigned.effective_type(), Some(SensorType::Camera));
    assigned.clone_from(&original);
    original_magnetometer_values(&original, "a_frame", "new_frame_id");
    original_magnetometer_values(&assigned, "a_frame", "new_frame_id");
    // The native original repeats sensor.FrameId() after assignment; its
    // destination-frame check here strengthens, rather than replaces, that row.
    assert_eq!(original.frame_id, "new_frame_id");
}

#[test]
fn original_dom_sensor_copy_assignment_after_move() {
    let mut sensor1 = magnetometer(0., "frame1", "new_frame_id");
    let mut sensor2 = sensor("camera");
    sensor2.pose.transform.translation = [4., 5., 6.];
    sensor2.pose.relative_to = "frame2".into();
    sensor2.frame_id = "new_frame_id2".into();

    // The native code moves sensor1 into tmp, then copies sensor2 into sensor1
    // and tmp into sensor2. Rust's replace transfers the owned old value.
    let tmp = std::mem::replace(&mut sensor1, sensor2.clone());
    sensor2 = tmp;

    assert_eq!(sensor1.effective_type(), Some(SensorType::Camera));
    assert_eq!(sensor1.pose.transform.translation, [4., 5., 6.]);
    assert_eq!(sensor1.pose.transform.rotation, [0., 0., 0., 1.]);
    assert_eq!(sensor1.pose.relative_to, "frame2");
    assert_eq!(sensor1.frame_id, "new_frame_id2");
    original_magnetometer_values(&sensor2, "frame1", "new_frame_id");
}
