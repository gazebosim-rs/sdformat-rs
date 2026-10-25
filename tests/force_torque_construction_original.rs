/*
 * Copyright (C) 2020 Open Source Robotics Foundation
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

use sdformat::{ForceTorque, ForceTorqueFrame, ForceTorqueMeasureDirection, Noise, NoiseKind};

#[test]
fn original_force_torque_construction_and_owned_copy_move() {
    let mut ft = ForceTorque::default();
    assert!(ft.source.is_none(), "owned default source is None");
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
    for axis in &mut ft.force {
        assert_eq!(*axis, default_noise);
        *axis = noise;
        assert_eq!(*axis, noise);
    }
    for axis in &mut ft.torque {
        assert_eq!(*axis, default_noise);
        *axis = noise;
        assert_eq!(*axis, noise);
    }
    assert_eq!(ft.frame, ForceTorqueFrame::Child);
    ft.frame = ForceTorqueFrame::Parent;
    assert_eq!(ft.frame, ForceTorqueFrame::Parent);
    assert_eq!(
        ft.measure_direction,
        ForceTorqueMeasureDirection::ChildToParent
    );
    ft.measure_direction = ForceTorqueMeasureDirection::ParentToChild;
    assert_eq!(
        ft.measure_direction,
        ForceTorqueMeasureDirection::ParentToChild
    );

    let mut ft2 = ft.clone();
    assert_eq!(ft2, ft, "copy construction");
    let mut ft3 = ForceTorque::default();
    assert_ne!(ft3, ft);
    ft3.clone_from(&ft);
    assert_eq!(ft3, ft, "copy assignment");

    let ft4 = ft;
    assert_eq!(ft4, ft2, "move construction");
    ft = ft4.clone();
    assert_eq!(ft, ft2, "copy assignment after move");

    let mut ft5 = ForceTorque::default();
    assert_ne!(ft5, ft3);
    ft5 = ft2;
    assert_eq!(ft5, ft3, "move assignment");
    ft2 = ft5.clone();
    assert_eq!(ft2, ft3, "copy assignment after move assignment");

    let ft6 = ForceTorque::default();
    assert_ne!(ft6, ft3);
}
