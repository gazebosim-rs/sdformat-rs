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
// Supported ToElement observations; generic Element identity remains unported.
use sdformat::*;
use std::process::Command;
fn vector(actual: [f64; 3], expected: [f64; 3]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!((actual - expected).abs() <= 1e-3);
    }
}
fn original_body() {
    let noise = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };
    let value = Imu {
        angular_velocity: [noise; 3],
        linear_acceleration: [noise; 3],
        enable_orientation: false,
        localization: "NED".into(),
        custom_rpy: [0., 0., 1.],
        custom_rpy_parent_frame: "other_frame".into(),
        gravity_dir_x: [0.; 3],
        gravity_dir_x_parent_frame: "my_frame".into(),
    };
    let output = value.to_xml();
    assert!(output.is_ok());
    let xml = output.unwrap();
    assert!(!xml.is_empty());
    let loaded = load_imu_str(&xml, "imu.xml");
    assert!(loaded.is_ok());
    let mut loaded = loaded.unwrap();
    for actual in loaded
        .linear_acceleration
        .into_iter()
        .chain(loaded.angular_velocity)
    {
        assert_eq!(actual, noise);
    }
    vector(loaded.gravity_dir_x, [0.; 3]);
    assert_eq!(loaded.gravity_dir_x_parent_frame, "my_frame");
    vector(loaded.custom_rpy, [0., 0., 1.]);
    assert_eq!(loaded.custom_rpy_parent_frame, "other_frame");
    assert_eq!(loaded.localization, "NED");
    assert!(!loaded.enable_orientation);
    loaded.gravity_dir_x = [1., 2., 3.];
    let output = loaded.to_xml().unwrap();
    assert!(!output.is_empty());
    vector(
        load_imu_str(&output, "edited.xml").unwrap().gravity_dir_x,
        [1., 2., 3.],
    );
}
#[test]
fn original_imu_to_element_values_subset() {
    original_body();
}
#[test]
fn original_imu_to_element_error_output_subset() {
    if std::env::var_os("SDF_IMU_QUIET_CHILD").is_some() {
        original_body();
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "original_imu_to_element_error_output_subset",
            "--nocapture",
            "--color",
            "never",
        ])
        .env("SDF_IMU_QUIET_CHILD", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let prefix = "\nrunning 1 test\ntest original_imu_to_element_error_output_subset ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in ";
    let elapsed = stdout
        .strip_prefix(prefix)
        .and_then(|text| text.strip_suffix("s\n\n"))
        .expect(&stdout)
        .parse::<f64>()
        .unwrap();
    assert!(elapsed.is_finite() && elapsed >= 0.);
}
