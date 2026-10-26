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
// Pinned src/JointAxis_TEST.cc DOMJointAxis.ZeroNormVectorReturnsError.
use sdformat::{Axis, AxisDirectionError, NoResources, load_str};

fn axis() -> Axis {
    load_str(
        "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child><axis><xyz>0 0 1</xyz></axis></joint></model></sdf>",
        "joint-axis-direction.sdf",
        &NoResources,
    )
    .unwrap()
    .models[0]
    .joints[0]
    .axis
    .clone()
    .unwrap()
}

#[test]
fn original_dom_joint_axis_zero_norm_vector_returns_error() {
    let mut axis = axis();
    assert!(axis.try_set_xyz([1., 0., 0.]).is_ok());
    let error = axis.try_set_xyz([0., 0., 0.]).unwrap_err();
    assert_eq!(
        error.to_string(),
        "The norm of the xyz vector cannot be zero"
    );
    assert_eq!(axis.xyz, [1., 0., 0.]); // Supplemental: rejection is atomic.
}

#[test]
fn supplemental_direction_normalization_and_bounds() {
    let mut axis = axis();
    axis.try_set_xyz([0., 3., 4.]).unwrap();
    assert_eq!(axis.xyz, [0., 0.6, 0.8]);

    let prior = axis.xyz;
    assert_eq!(
        axis.try_set_xyz([f64::from(1e-6_f32), 0., 0.]),
        Err(AxisDirectionError::ZeroLength)
    );
    assert_eq!(axis.xyz, prior);
    assert_eq!(
        axis.try_set_xyz([f64::INFINITY, 0., 0.]),
        Err(AxisDirectionError::NonFinite)
    );
    assert_eq!(axis.xyz, prior);
    assert_eq!(
        AxisDirectionError::NonFinite.to_string(),
        "The xyz vector must be finite"
    );
}
