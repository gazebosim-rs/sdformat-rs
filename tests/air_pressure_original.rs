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

use sdformat::{AirPressure, ErrorCode, NoResources, Noise, NoiseKind, SensorConfig, load_str};

#[test]
fn original_air_pressure_construction() {
    let air = AirPressure::default();
    assert_eq!(air.pressure, Noise::default());
    assert_eq!(air.reference_altitude, 0.0);
}

#[test]
fn original_air_pressure_set_copy_and_move() {
    let mut air = AirPressure::default();
    assert_eq!(air.pressure, Noise::default());
    assert_eq!(air.reference_altitude, 0.0);
    let noise = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };
    air.pressure = noise;
    assert_eq!(air.pressure, noise);
    air.reference_altitude = 10.2;
    assert_eq!(air.reference_altitude, 10.2);

    let air2 = air.clone();
    assert_eq!(air, air2, "copy construction");
    let mut air3 = AirPressure::default();
    assert_ne!(air3, air);
    air3.clone_from(&air);
    assert_eq!(air, air3, "copy assignment");
    let air4 = air;
    assert_eq!(air2, air4, "move construction");
    air = air4.clone();
    assert_eq!(air2, air, "copy assignment after move");
    let mut air5 = AirPressure::default();
    assert_ne!(air5, air3);
    air5 = air2;
    assert_eq!(air3, air5, "move assignment");
    let air6 = AirPressure::default();
    assert_ne!(air3, air6);
}

fn sensor_xml(child: &str, kind: &str) -> String {
    format!(
        "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'>{child}</sensor></link></model></sdf>"
    )
}

#[test]
fn native_air_pressure_sensor_fragment_is_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let start = fixture
        .find("<sensor name=\"air_pressure_sensor\" type=\"air_pressure\">")
        .unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let exact_sensor = &fixture[start..end];
    let xml = format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{exact_sensor}</link></model></sdf>"
    );
    let sdf = load_str(&xml, "sensors.sdf#air_pressure_sensor", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "air_pressure_sensor");
    assert_eq!(sensor.kind, "air_pressure");
    assert!(matches!(sensor.config, SensorConfig::AirPressure(_)));
    assert_eq!(sensor.pose.transform.translation, [10., 20., 30.]);
    assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(!sensor.enable_metrics);
    assert!(sensor.imu().is_none());
    assert!(sensor.contact().is_none());
    assert!(sensor.force_torque().is_none());
    assert!(sensor.magnetometer().is_none());
    let air = sensor.air_pressure().unwrap();
    assert!(air.source.is_some());
    assert_eq!(air.pressure.mean, 3.4);
    assert_eq!(air.pressure.stddev, 5.6);
    assert_eq!(air.reference_altitude, 123.4);
    assert!(sensor.configuration_xml.starts_with("<air_pressure>"));

    let default = load_str(
        &sensor_xml("", "air_pressure"),
        "default-air-pressure.sdf",
        &NoResources,
    )
    .unwrap();
    let sensor = &default.models[0].links[0].sensors[0];
    assert_eq!(sensor.air_pressure().unwrap(), &AirPressure::default());
    assert!(sensor.configuration_xml.is_empty());
    let empty = load_str(
        &sensor_xml("<air_pressure><pressure/></air_pressure>", "air_pressure"),
        "empty-air-pressure.sdf",
        &NoResources,
    )
    .unwrap();
    let air = empty.models[0].links[0].sensors[0].air_pressure().unwrap();
    assert_eq!(air.pressure, Noise::default());
    assert_eq!(air.reference_altitude, 0.0);
    assert!(air.source.is_some());
    let imu = load_str(&sensor_xml("", "imu"), "default-imu.sdf", &NoResources).unwrap();
    assert!(imu.models[0].links[0].sensors[0].air_pressure().is_none());
    for (kind, child) in [("imu", "air_pressure"), ("air_pressure", "imu")] {
        let xml = sensor_xml(&format!("<{child}/>"), kind);
        assert_eq!(
            load_str(&xml, "mismatched-air-pressure.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let invalid = sensor_xml("<air_pressure><unknown/></air_pressure>", "air_pressure");
    assert_eq!(
        load_str(&invalid, "invalid-air-pressure.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}
