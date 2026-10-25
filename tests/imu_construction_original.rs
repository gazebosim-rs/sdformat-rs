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

use sdformat::{Imu, Noise, NoiseKind};

#[test]
fn original_imu_construction_and_owned_copy_move() {
    let mut imu = Imu::default();
    let default_noise = Noise::default();
    let noise = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };
    for axis in &mut imu.linear_acceleration {
        assert_eq!(*axis, default_noise);
        *axis = noise;
        assert_eq!(*axis, noise);
    }
    for axis in &mut imu.angular_velocity {
        assert_eq!(*axis, default_noise);
        *axis = noise;
        assert_eq!(*axis, noise);
    }
    assert_eq!(imu.gravity_dir_x, [1., 0., 0.]);
    imu.gravity_dir_x = [0.; 3];
    assert_eq!(imu.gravity_dir_x, [0.; 3]);
    assert!(imu.gravity_dir_x_parent_frame.is_empty());
    imu.gravity_dir_x_parent_frame = "my_frame".into();
    assert_eq!(imu.gravity_dir_x_parent_frame, "my_frame");
    assert_eq!(imu.custom_rpy, [0.; 3]);
    imu.custom_rpy = [0., 0., 1.];
    assert_eq!(imu.custom_rpy, [0., 0., 1.]);
    assert!(imu.custom_rpy_parent_frame.is_empty());
    imu.custom_rpy_parent_frame = "other_frame".into();
    assert_eq!(imu.custom_rpy_parent_frame, "other_frame");
    assert_eq!(imu.localization, "CUSTOM");
    imu.localization = "NED".into();
    assert_eq!(imu.localization, "NED");
    assert!(imu.enable_orientation);
    imu.enable_orientation = false;
    assert!(!imu.enable_orientation);

    let mut imu2 = imu.clone();
    assert_eq!(imu2, imu, "copy construction");
    let mut imu3 = Imu::default();
    assert_ne!(imu3, imu);
    imu3.clone_from(&imu);
    assert_eq!(imu3, imu, "copy assignment");
    let imu4 = imu;
    assert_eq!(imu2, imu4, "move construction");
    imu = imu4.clone();
    assert_eq!(imu2, imu, "copy assignment after move");
    let mut imu5 = Imu::default();
    assert_ne!(imu5, imu3);
    imu5 = imu2;
    assert_eq!(imu3, imu5, "move assignment");
    imu2 = imu5.clone();
    assert_eq!(imu3, imu2, "copy assignment after move assignment");
    let imu6 = Imu::default();
    assert_ne!(imu3, imu6);
}
