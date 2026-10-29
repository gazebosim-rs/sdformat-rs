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
 */
// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{NoResources, load_str};

fn double_eq(actual: f64, expected: f64) {
    assert!(
        actual.to_bits().abs_diff(expected.to_bits()) <= 4,
        "{actual} != {expected}"
    );
}
fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-3, "{actual} != {expected}");
}
#[test]
fn original_dom_joint_screw_thread_pitch() {
    let sdf = load_str(
        include_str!("fixtures/joint_screw_thread_pitch.sdf"),
        "joint_screw_thread_pitch.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().unwrap();
    assert_eq!(model.links.len(), 2);
    assert!(model.link_by_name("child_link").is_some());
    assert!(model.link_by_name("parent_link").is_some());
    assert_eq!(model.joints.len(), 5);
    let joint = |name| model.joint_by_name(name).unwrap();
    for name in [
        "default_param",
        "both_params",
        "new_param",
        "old_param",
        "param_precedence",
    ] {
        assert!(model.joint_by_name(name).is_some());
    }
    double_eq(joint("default_param").screw_thread_pitch, 1.);
    double_eq(joint("both_params").screw_thread_pitch, 0.5);
    double_eq(joint("new_param").screw_thread_pitch, 0.5);
    near(joint("old_param").screw_thread_pitch, 0.5);
    double_eq(joint("param_precedence").screw_thread_pitch, 0.5);
    double_eq(
        joint("default_param").legacy_thread_pitch(),
        -std::f64::consts::TAU,
    );
    near(joint("both_params").legacy_thread_pitch(), -12.566);
    near(joint("new_param").legacy_thread_pitch(), -12.566);
    double_eq(joint("old_param").legacy_thread_pitch(), -12.566);
    near(joint("param_precedence").legacy_thread_pitch(), -12.566);
}
