/*
 * Copyright (C) 2019 Open Source Robotics Foundation
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

use sdformat::{Altimeter, ErrorCode, NoResources, Noise, NoiseKind, SensorConfig, load_str};

#[test]
fn original_altimeter_construction() {
    let alt = Altimeter::default();
    let default_noise = Noise::default();
    assert_eq!(alt.vertical_position, default_noise);
    assert_eq!(alt.vertical_velocity, default_noise);
}

#[test]
fn original_altimeter_set_copy_and_move() {
    let mut alt = Altimeter::default();
    let default_noise = Noise::default();
    assert_eq!(alt.vertical_position, default_noise);
    assert_eq!(alt.vertical_velocity, default_noise);
    let noise = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };
    alt.vertical_position = noise;
    assert_eq!(alt.vertical_position, noise);
    assert_eq!(alt.vertical_velocity, default_noise);
    alt.vertical_velocity = noise;
    assert_eq!(alt.vertical_position, noise);
    assert_eq!(alt.vertical_velocity, noise);

    let mut alt2 = alt.clone();
    assert_eq!(alt, alt2, "copy construction");
    let mut alt3 = Altimeter::default();
    assert_ne!(alt3, alt);
    alt3.clone_from(&alt);
    assert_eq!(alt, alt3, "copy assignment");
    let alt4 = alt;
    assert_eq!(alt2, alt4, "move construction");
    alt = alt4.clone();
    assert_eq!(alt2, alt, "copy assignment after move");
    let mut alt5 = Altimeter::default();
    assert_ne!(alt5, alt3);
    alt5 = alt2;
    assert_eq!(alt3, alt5, "move assignment");
    alt2 = alt5.clone();
    assert_eq!(alt3, alt2, "copy assignment after move assignment");
    let mut alt6 = Altimeter::default();
    assert_ne!(alt3, alt6);
    alt6.vertical_position = alt3.vertical_position;
    assert_ne!(alt3, alt6, "velocity remains different");
}

fn sensor_xml(child: &str, kind: &str) -> String {
    format!(
        "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'>{child}</sensor></link></model></sdf>"
    )
}

#[test]
fn native_altimeter_sensor_fragment_is_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let start = fixture
        .find("<sensor name=\"altimeter_sensor\" type=\"altimeter\">")
        .unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let exact_sensor = &fixture[start..end];
    let xml = format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{exact_sensor}</link></model></sdf>"
    );
    let sdf = load_str(&xml, "sensors.sdf#altimeter_sensor", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "altimeter_sensor");
    assert_eq!(sensor.kind, "altimeter");
    assert!(matches!(sensor.config, SensorConfig::Altimeter(_)));
    assert_eq!(sensor.pose.transform.translation, [0.; 3]);
    assert!(!sensor.enable_metrics);
    assert!(sensor.imu().is_none());
    assert!(sensor.contact().is_none());
    assert!(sensor.force_torque().is_none());
    assert!(sensor.magnetometer().is_none());
    let alt = sensor.altimeter().unwrap();
    assert!(alt.source.is_some());
    assert_eq!(alt.vertical_position.mean, 0.1);
    assert_eq!(alt.vertical_position.stddev, 0.2);
    assert_eq!(alt.vertical_velocity.mean, 2.3);
    assert_eq!(alt.vertical_velocity.stddev, 4.5);
    assert!(sensor.configuration_xml.starts_with("<altimeter>"));

    let default = load_str(
        &sensor_xml("", "altimeter"),
        "default-altimeter.sdf",
        &NoResources,
    )
    .unwrap();
    let sensor = &default.models[0].links[0].sensors[0];
    assert_eq!(sensor.altimeter().unwrap(), &Altimeter::default());
    assert!(sensor.configuration_xml.is_empty());
    let empty = load_str(
        &sensor_xml("<altimeter><vertical_position/></altimeter>", "altimeter"),
        "empty-altimeter.sdf",
        &NoResources,
    )
    .unwrap();
    let alt = empty.models[0].links[0].sensors[0].altimeter().unwrap();
    assert_eq!(alt.vertical_position, Noise::default());
    assert_eq!(alt.vertical_velocity, Noise::default());
    assert!(alt.source.is_some());
    let imu = load_str(&sensor_xml("", "imu"), "default-imu.sdf", &NoResources).unwrap();
    assert!(imu.models[0].links[0].sensors[0].altimeter().is_none());

    for (kind, child) in [("imu", "altimeter"), ("altimeter", "imu")] {
        let xml = sensor_xml(&format!("<{child}/>"), kind);
        assert_eq!(
            load_str(&xml, "mismatched-altimeter.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let invalid = sensor_xml("<altimeter><unknown/></altimeter>", "altimeter");
    assert_eq!(
        load_str(&invalid, "invalid-altimeter.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}
