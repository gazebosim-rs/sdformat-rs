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

use sdformat::{ErrorCode, Magnetometer, NoResources, Noise, NoiseKind, SensorConfig, load_str};

#[test]
fn original_magnetometer_construction() {
    let mag = Magnetometer::default();
    let default_noise = Noise::default();
    assert_eq!(mag.noise[0], default_noise);
    assert_eq!(mag.noise[1], default_noise);
    assert_eq!(mag.noise[2], default_noise);
}

#[test]
fn original_magnetometer_set_copy_and_move() {
    let mut mag = Magnetometer::default();
    let default_noise = Noise::default();
    assert_eq!(mag.noise[0], default_noise);
    assert_eq!(mag.noise[1], default_noise);
    assert_eq!(mag.noise[2], default_noise);
    let noise = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };

    mag.noise[0] = noise;
    assert_eq!(mag.noise[0], noise);
    assert_eq!(mag.noise[1], default_noise);
    assert_eq!(mag.noise[2], default_noise);
    mag.noise[1] = noise;
    assert_eq!(mag.noise[0], noise);
    assert_eq!(mag.noise[1], noise);
    assert_eq!(mag.noise[2], default_noise);
    mag.noise[2] = noise;
    assert_eq!(mag.noise[0], noise);
    assert_eq!(mag.noise[1], noise);
    assert_eq!(mag.noise[2], noise);

    let mag2 = mag.clone();
    assert_eq!(mag, mag2, "copy construction");
    let mut mag3 = Magnetometer::default();
    assert_ne!(mag3, mag);
    mag3.clone_from(&mag);
    assert_eq!(mag, mag3, "copy assignment");
    let mag4 = mag;
    assert_eq!(mag2, mag4, "move construction");
    let mut mag5 = Magnetometer::default();
    assert_ne!(mag5, mag3);
    mag5 = mag2;
    assert_eq!(mag3, mag5, "move assignment");
    let mag6 = Magnetometer::default();
    assert_ne!(mag3, mag6);

    let mut mag7 = Magnetometer::default();
    mag7.noise[0] = default_noise;
    let mut mag8 = Magnetometer::default();
    mag8.noise[0] = noise;
    let tmp = mag7;
    mag7 = mag8.clone();
    mag8 = tmp;
    assert_eq!(mag7.noise[0], noise);
    assert_eq!(mag8.noise[0], default_noise);
}

#[test]
fn native_magnetometer_sensor_fragment_is_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let start = fixture
        .find("<sensor name=\"magnetometer_sensor\" type=\"magnetometer\">")
        .unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let exact_sensor = &fixture[start..end];
    let xml = format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{exact_sensor}</link></model></sdf>"
    );
    let sdf = load_str(&xml, "sensors.sdf#magnetometer_sensor", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "magnetometer_sensor");
    assert_eq!(sensor.kind, "magnetometer");
    assert!(matches!(sensor.config, SensorConfig::Magnetometer(_)));
    assert_eq!(sensor.pose.transform.translation, [10., 11., 12.]);
    assert!(!sensor.enable_metrics);
    assert!(sensor.imu().is_none());
    assert!(sensor.contact().is_none());
    assert!(sensor.force_torque().is_none());
    let mag = sensor.magnetometer().unwrap();
    assert!(mag.source.is_some());
    assert_eq!(mag.noise[0].mean, 0.1);
    assert_eq!(mag.noise[0].stddev, 0.2);
    assert_eq!(mag.noise[1].mean, 1.2);
    assert_eq!(mag.noise[1].stddev, 2.3);
    assert_eq!(mag.noise[2].mean, 3.4);
    assert_eq!(mag.noise[2].stddev, 5.6);
    assert!(sensor.configuration_xml.starts_with("<magnetometer>"));

    let default = load_str(
        "<sdf version='1.10'><model name='m'><link name='l'><sensor name='mag' type='magnetometer'/></link></model></sdf>",
        "default-magnetometer.sdf",
        &NoResources,
    )
    .unwrap();
    let sensor = &default.models[0].links[0].sensors[0];
    assert_eq!(sensor.magnetometer().unwrap(), &Magnetometer::default());
    assert!(sensor.configuration_xml.is_empty());
    assert!(sensor.imu().is_none());
    assert!(sensor.magnetometer().is_some());

    let imu = load_str(
        "<sdf version='1.10'><model name='m'><link name='l'><sensor name='imu' type='imu'/></link></model></sdf>",
        "default-imu.sdf",
        &NoResources,
    )
    .unwrap();
    assert!(imu.models[0].links[0].sensors[0].magnetometer().is_none());

    for (kind, child) in [("imu", "magnetometer"), ("magnetometer", "imu")] {
        let xml = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'><{child}/></sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "mismatched-magnetometer.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}
