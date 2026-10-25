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
// Pinned src/Sensor_TEST.cc DOMSensor.EnableMetrics, lines 339-364.
use sdformat::{ErrorCode, NoResources, Sensor, SensorConfig, SensorType, load_str};

const ORIGINAL_DOCUMENT: &str = concat!(
    "<sdf version='1.5'>",
    "  <model name='test_model'>",
    "    <sensor name='test_sensor' type='none'>",
    "      <enable_metrics>true</enable_metrics>",
    "    </sensor>",
    "  </model>",
    "</sdf>",
);

#[test]
fn original_dom_sensor_enable_metrics() {
    let mut sensor = Sensor::default();
    assert!(!sensor.enable_metrics);

    // Native SDF first obtains this exact Element from the inline document,
    // then calls Sensor::Load on the element rather than Root::Load.
    let start = ORIGINAL_DOCUMENT.find("<sensor ").unwrap();
    let end = ORIGINAL_DOCUMENT.find("</sensor>").unwrap() + "</sensor>".len();
    let element = &ORIGINAL_DOCUMENT[start..end];
    // Native Load reports ATTRIBUTE_INVALID for `type='none'` after setting
    // enable_metrics; the original ignores its returned error list.
    assert_eq!(
        sensor
            .load_element_str(element, "sensor-metrics.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
    assert!(sensor.enable_metrics);
    sensor.enable_metrics = false;
    assert!(!sensor.enable_metrics);
}

#[test]
fn standalone_none_sensor_reports_error_after_partial_mutation() {
    let element =
        "<sensor name='test_sensor' type='none'><enable_metrics>true</enable_metrics></sensor>";
    let mut sensor = Sensor::default();
    assert_eq!(
        sensor
            .load_element_str(element, "standalone.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
    assert_eq!(sensor.kind, "none");
    assert_eq!(sensor.effective_type(), None);
    assert_eq!(
        SensorType::from_type_str(&sensor.kind),
        Some(SensorType::None)
    );
    assert!(matches!(sensor.config, SensorConfig::Unconfigured));
    assert!(sensor.enable_metrics);

    let document = "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='none'/></link></model></sdf>";
    assert_eq!(
        load_str(document, "full.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    assert_eq!(
        sensor
            .load_element_str("<model name='m'/>", "wrong.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Xml
    );
    assert_eq!(
        sensor
            .load_element_str("<sensor", "broken.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Xml
    );

    sensor
        .load_element_str(
            "<sensor name='imu_sensor' type='imu'><enable_metrics>true</enable_metrics><imu/></sensor>",
            "valid.sdf",
            &NoResources,
        )
        .unwrap();
    assert_eq!(sensor.name, "imu_sensor");
    assert_eq!(sensor.kind, "imu");
    assert!(sensor.enable_metrics);
}
