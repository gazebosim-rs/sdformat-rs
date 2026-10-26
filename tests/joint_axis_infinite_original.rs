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
// Pinned test/integration/joint_axis_dom.cc DOMJointAxis.InfiniteLimits.
use sdformat::{NoResources, load_str};

#[test]
fn original_dom_joint_axis_infinite_limits() {
    let sdf = load_str(
        include_str!("fixtures/joint_axis_infinite_limits.sdf"),
        "joint_axis_infinite_limits.sdf",
        &NoResources,
    )
    .expect("native Root.Load has no errors");
    let model = sdf.models.first().expect("native Root.Model is present");
    assert_eq!(model.name, "joint_axis_infinite_limits");

    let infinity = f64::INFINITY;
    for (name, expected) in [
        (
            "default_joint_limits",
            [-infinity, infinity, infinity, infinity],
        ),
        ("finite_joint_limits", [-1.5, 1.5, 5.5, 2.5]),
        (
            "infinite_joint_limits_inf",
            [-infinity, infinity, infinity, infinity],
        ),
        (
            "infinite_joint_limits_neg",
            [-infinity, infinity, infinity, infinity],
        ),
    ] {
        let joint = model
            .joint_by_name(name)
            .expect("native JointByName succeeds");
        let axis = joint.axis(0).expect("native Axis(0) is present");
        assert_eq!(
            [axis.lower, axis.upper, axis.effort, axis.velocity],
            expected,
            "{name} limits",
        );
    }
}
