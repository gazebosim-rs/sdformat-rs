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
use sdformat::{NoResources, load_str};

// The twelve dynamics/stop default and setter observations only. Element,
// XYZ/resolve validation, bounds and mimic observations stay explicitly pending.
#[test]
fn original_joint_axis_construction_dynamics_and_stop_values() {
    let mut sdf = load_str("<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child><axis/></joint></model></sdf>", "defaults", &NoResources).unwrap();
    let axis = sdf.models[0].joints[0].axis.as_mut().unwrap();
    assert_eq!(axis.dynamics.damping, 0.0);
    assert_eq!(axis.dynamics.friction, 0.0);
    assert_eq!(axis.dynamics.spring_reference, 0.0);
    assert_eq!(axis.dynamics.spring_stiffness, 0.0);
    assert_eq!(axis.stiffness, 1e8);
    assert_eq!(axis.dissipation, 1.0);
    axis.dynamics.damping = 0.2;
    assert_eq!(axis.dynamics.damping, 0.2);
    axis.dynamics.friction = 1.3;
    assert_eq!(axis.dynamics.friction, 1.3);
    axis.dynamics.spring_reference = 2.4;
    assert_eq!(axis.dynamics.spring_reference, 2.4);
    axis.dynamics.spring_stiffness = -1.2;
    assert_eq!(axis.dynamics.spring_stiffness, -1.2);
    axis.stiffness = 1e2;
    assert_eq!(axis.stiffness, 1e2);
    axis.dissipation = 1.5;
    assert_eq!(axis.dissipation, 1.5);
}
