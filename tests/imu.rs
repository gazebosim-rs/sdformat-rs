// SPDX-License-Identifier: Apache-2.0
// Copyright 2018 Open Source Robotics Foundation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Assertion subsets ported from sdformat/test/integration/link_dom.cc (OSRF,
// Apache-2.0). The surrounding 26 sensor families remain pending.
use sdformat::*;

fn sensor(imu: &str) -> Sensor {
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='imu'>{imu}</sensor></link></model></sdf>"
    );
    let mut sdf = load_str(&xml, "imu.sdf", &NoResources).unwrap();
    sdf.models.remove(0).links.remove(0).sensors.remove(0)
}

#[test]
fn link_dom_sensors_original_imu_assertions() {
    let original = include_str!("fixtures/sensors.sdf");
    // The full fixture has an undeclared prefix in another sensor family.
    // Keep the exact IMU subtree bytes; no content or assertions are rewritten.
    let start = original.find("<sensor name=\"imu_sensor\"").unwrap();
    let end = start + original[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let xml = format!(
        "<sdf version='1.12'><model name='model'><link name='link'>{}</link></model></sdf>",
        &original[start..end]
    );
    let sdf = load_str(&xml, "sensors.sdf#imu_sensor", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "imu_sensor");
    assert_eq!(sensor.kind, "imu");
    assert_eq!(sensor.pose.transform.translation, [4., 5., 6.]);
    assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(!sensor.enable_metrics);
    let imu = sensor.imu().unwrap();
    for (actual, expected) in imu
        .linear_acceleration
        .iter()
        .chain(&imu.angular_velocity)
        .zip([
            [0., 0.1, 0.2, 1.],
            [1., 1.1, 1.2, 2.],
            [2., 2.1, 2.2, 3.],
            [3., 3.1, 4.2, 4.],
            [4., 4.1, 5.2, 5.],
            [5., 5.1, 6.2, 6.],
        ])
    {
        assert_eq!(
            [
                actual.mean,
                actual.stddev,
                actual.dynamic_bias_stddev,
                actual.dynamic_bias_correlation_time
            ],
            expected
        );
        assert_eq!(actual.kind, NoiseKind::Gaussian);
    }
    assert_eq!(imu.localization, "ENU");
    assert_eq!(imu.custom_rpy_parent_frame, "linka");
    assert_eq!(imu.custom_rpy, [0., 1., 0.]);
    assert_eq!(imu.gravity_dir_x_parent_frame, "linkb");
    assert_eq!(imu.gravity_dir_x, [0., 0., 1.]);
    assert!(!imu.enable_orientation);
}

#[test]
fn typed_defaults_and_all_noise_fields() {
    let empty = sensor("");
    let explicit = sensor("<imu/>");
    assert_eq!(empty.imu().unwrap(), explicit.imu().unwrap());
    let imu = empty.imu().unwrap();
    assert!(imu.enable_orientation);
    assert_eq!(imu.localization, "CUSTOM");
    assert_eq!(imu.custom_rpy, [0.; 3]);
    assert_eq!(imu.gravity_dir_x, [1., 0., 0.]);
    assert_eq!(imu.custom_rpy_parent_frame, "");
    assert_eq!(imu.gravity_dir_x_parent_frame, "");
    for noise in imu.angular_velocity.iter().chain(&imu.linear_acceleration) {
        assert_eq!(*noise, Noise::default());
        assert_eq!(noise.kind, NoiseKind::None);
    }
    let all = sensor(
        "<enable_metrics>1</enable_metrics><imu><angular_velocity><x><noise type='GaUsSiaN_QUANTIZED'><mean>1.2</mean><stddev>2.3</stddev><bias_mean>4.5</bias_mean><bias_stddev>6.7</bias_stddev><precision>8.9</precision><dynamic_bias_stddev>10.11</dynamic_bias_stddev><dynamic_bias_correlation_time>12.13</dynamic_bias_correlation_time></noise></x><y/><z><noise type='none'/></z></angular_velocity><orientation_reference_frame><localization>BOGUS</localization><custom_rpy>0 0 1</custom_rpy><grav_dir_x>0 0 0</grav_dir_x></orientation_reference_frame></imu>",
    );
    assert!(all.enable_metrics);
    assert_eq!(all.imu().unwrap().localization, "BOGUS");
    assert_eq!(all.imu().unwrap().custom_rpy, [0., 0., 1.]);
    assert_eq!(all.imu().unwrap().gravity_dir_x, [0.; 3]);
    assert_eq!(
        all.imu().unwrap().angular_velocity[0],
        Noise {
            kind: NoiseKind::GaussianQuantized,
            mean: 1.2,
            stddev: 2.3,
            bias_mean: 4.5,
            bias_stddev: 6.7,
            precision: 8.9,
            dynamic_bias_stddev: 10.11,
            dynamic_bias_correlation_time: 12.13
        }
    );
    assert_eq!(all.imu().unwrap().angular_velocity[1], Noise::default());
    assert_eq!(all.imu().unwrap().angular_velocity[2], Noise::default());
    let original = all.imu().unwrap().clone();
    let mut copied = Imu::default();
    copied.clone_from(&original);
    assert_eq!(original, copied);
    assert_ne!(original, Imu::default());
}

#[test]
fn malformed_configuration_is_not_opaque() {
    for (content, code) in [
        ("<unknown/>", ErrorCode::Unsupported),
        (
            "<enable_orientation>maybe</enable_orientation>",
            ErrorCode::InvalidValue,
        ),
        (
            "<angular_velocity><x><unknown/></x></angular_velocity>",
            ErrorCode::Unsupported,
        ),
        (
            "<angular_velocity><w/></angular_velocity>",
            ErrorCode::Unsupported,
        ),
        (
            "<angular_velocity><x><noise type='bad'/></x></angular_velocity>",
            ErrorCode::InvalidValue,
        ),
        (
            "<angular_velocity><x><noise type='gaussian'><stddev>-1</stddev></noise></x></angular_velocity>",
            ErrorCode::InvalidValue,
        ),
        (
            "<angular_velocity><x><noise type='gaussian'><bias_stddev>-1</bias_stddev></noise></x></angular_velocity>",
            ErrorCode::InvalidValue,
        ),
        (
            "<angular_velocity><x><noise type='gaussian'><dynamic_bias_stddev>-1</dynamic_bias_stddev></noise></x></angular_velocity>",
            ErrorCode::InvalidValue,
        ),
        (
            "<angular_velocity><x><noise type='gaussian'><mean>NaN</mean></noise></x></angular_velocity>",
            ErrorCode::InvalidValue,
        ),
        (
            "<angular_velocity><x><noise type='gaussian'><extra>1</extra></noise></x></angular_velocity>",
            ErrorCode::Unsupported,
        ),
        (
            "<orientation_reference_frame><extra/></orientation_reference_frame>",
            ErrorCode::Unsupported,
        ),
        (
            "<orientation_reference_frame><custom_rpy>0 1</custom_rpy></orientation_reference_frame>",
            ErrorCode::InvalidValue,
        ),
        (
            "<orientation_reference_frame><grav_dir_x parent_frame='world'>0 NaN 1</grav_dir_x></orientation_reference_frame>",
            ErrorCode::InvalidValue,
        ),
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='imu'><imu>{content}</imu></sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "bad-imu", &NoResources).unwrap_err().code,
            code,
            "{content}"
        );
    }
}
