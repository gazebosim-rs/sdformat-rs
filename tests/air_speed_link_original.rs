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

use sdformat::{AirSpeed, ErrorCode, NoResources, Noise, SensorConfig, load_str};

fn sensor_xml(child: &str, kind: &str) -> String {
    format!(
        "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'>{child}</sensor></link></model></sdf>"
    )
}

#[test]
fn native_air_speed_link_sensor_fragment_is_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let start = fixture
        .find("<sensor name=\"air_speed_sensor\" type=\"air_speed\">")
        .unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let exact_sensor = &fixture[start..end];
    let xml = format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{exact_sensor}</link></model></sdf>"
    );
    let sdf = load_str(&xml, "sensors.sdf#air_speed_sensor", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "air_speed_sensor");
    assert_eq!(sensor.kind, "air_speed");
    assert!(matches!(sensor.config, SensorConfig::AirSpeed(_)));
    assert_eq!(sensor.pose.transform.translation, [11., 22., 33.]);
    assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(!sensor.enable_metrics);
    let speed = sensor.air_speed().unwrap();
    assert!(speed.source.is_some());
    assert_eq!(speed.pressure.mean, 0.);
    assert_eq!(speed.pressure.stddev, 0.01);
    assert!(sensor.configuration_xml.starts_with("<air_speed>"));

    let default = load_str(
        &sensor_xml("", "air_speed"),
        "default-air-speed.sdf",
        &NoResources,
    )
    .unwrap();
    let sensor = &default.models[0].links[0].sensors[0];
    assert_eq!(sensor.air_speed().unwrap(), &AirSpeed::default());
    assert!(sensor.configuration_xml.is_empty());
    let empty = load_str(
        &sensor_xml("<air_speed><pressure/></air_speed>", "air_speed"),
        "empty-air-speed.sdf",
        &NoResources,
    )
    .unwrap();
    let speed = empty.models[0].links[0].sensors[0].air_speed().unwrap();
    assert_eq!(speed.pressure, Noise::default());
    assert!(speed.source.is_some());
    let imu = load_str(&sensor_xml("", "imu"), "default-imu.sdf", &NoResources).unwrap();
    assert!(imu.models[0].links[0].sensors[0].air_speed().is_none());
    for (kind, child) in [("imu", "air_speed"), ("air_speed", "imu")] {
        let xml = sensor_xml(&format!("<{child}/>"), kind);
        assert_eq!(
            load_str(&xml, "mismatched-air-speed.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let invalid = sensor_xml("<air_speed><unknown/></air_speed>", "air_speed");
    assert_eq!(
        load_str(&invalid, "invalid-air-speed.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}
