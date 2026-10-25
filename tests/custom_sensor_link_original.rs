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

use sdformat::{NoResources, SensorType, load_str};

#[test]
fn native_link_custom_extension_fragment_retains_element_and_integer_value() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let start = fixture
        .find("<sensor name=\"custom_sensor\" type=\"custom\">")
        .unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let xml = format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{}</link></model></sdf>",
        &fixture[start..end]
    );
    let sdf = load_str(&xml, "sensors.sdf#custom", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "custom_sensor");
    assert_eq!(sensor.effective_type(), Some(SensorType::Custom));
    let custom = sensor.custom().unwrap();
    assert_eq!(custom.element_name, "my_namespace:custom_sensor");
    assert_eq!(
        custom
            .parameter_text("some_param")
            .unwrap()
            .parse::<i32>()
            .unwrap(),
        123
    );
    assert_eq!(custom.parameter_text("absent"), None);
    assert!(
        sensor
            .configuration_xml
            .starts_with("<my_namespace:custom_sensor>")
    );
    assert!(
        sensor
            .configuration_xml
            .ends_with("</my_namespace:custom_sensor>")
    );
    assert!(sensor.imu().is_none());
    assert!(sensor.contact().is_none());
    assert!(sensor.force_torque().is_none());
    assert!(sensor.magnetometer().is_none());
    assert!(sensor.altimeter().is_none());
    assert!(sensor.air_pressure().is_none());
    assert!(sensor.navsat().is_none());
    assert!(sensor.air_speed().is_none());
    assert!(sensor.lidar().is_none());
    assert!(sensor.camera().is_none());
}
