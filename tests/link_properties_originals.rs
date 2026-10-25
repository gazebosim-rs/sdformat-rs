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
// Original assertions: link_dom.cc InertialDoublePendulum and joint_dom.cc DoublePendulum.
use sdformat::{NoResources, Transform, load_str};

fn double(actual: f64, expected: f64) {
    let ordered = |value: f64| {
        let bits = value.to_bits();
        if bits >> 63 == 1 {
            (!bits).wrapping_add(1)
        } else {
            bits | (1_u64 << 63)
        }
    };
    assert!(actual.is_finite() && expected.is_finite());
    assert!(
        ordered(actual).abs_diff(ordered(expected)) <= 4,
        "{actual} != {expected}"
    );
}
fn pose(actual: Transform, expected: [f64; 6]) {
    let q = glam::DQuat::from_euler(glam::EulerRot::ZYX, expected[5], expected[4], expected[3]);
    for (a, b) in actual
        .translation
        .into_iter()
        .zip(expected[..3].iter().copied())
    {
        assert!((a - b).abs() <= 1e-3, "{a} != {b}");
    }
    for (a, b) in actual.rotation.into_iter().zip(q.to_array()) {
        assert!((a - b).abs() <= 1e-3, "{a} != {b}");
    }
}

#[test]
// Preserve the upstream fixture/assertion's literal rather than substitute PI/2.
#[allow(clippy::approx_constant)]
fn original_link_inertial_double_pendulum() {
    // Typed loading validates each rigid inertia before returning it. Together
    // with this fixture's exact mass/matrix values, this retains its true IsValid
    // observation, as in the reviewed InertialComplete port; no generic math API.
    let checked = load_str(
        include_str!("fixtures/double_pendulum.sdf"),
        "double_pendulum.sdf",
        &NoResources,
    );
    assert!(checked.is_ok());
    let sdf = checked.unwrap();
    let model = sdf.models.first().unwrap();
    let base = model.links.first().unwrap();
    pose(base.pose.transform, [0.; 6]);
    assert_eq!(base.pose.relative_to, "");
    double(base.inertial.mass, 100.0);
    for (actual, expected) in base
        .inertial
        .inertia
        .into_iter()
        .zip([1., 1., 1., 0., 0., 0.])
    {
        double(actual, expected);
    }
    assert!(base.inertial.fluid_added_mass.is_none());
    let upper = model.links.get(1).unwrap();
    pose(upper.pose.transform, [0., 0., 2.1, -1.5708, 0., 0.]);
    assert_eq!(upper.pose.relative_to, "");
    assert!(upper.properties.enable_wind);
    assert!(upper.properties.kinematic);
    double(upper.inertial.mass, 1.0);
    for (actual, expected) in upper
        .inertial
        .inertia
        .into_iter()
        .zip([1., 1., 1., 0., 0., 0.])
    {
        double(actual, expected);
    }
    for (actual, expected) in upper
        .inertial
        .pose
        .transform
        .translation
        .into_iter()
        .zip([0., 0., 0.5])
    {
        double(actual, expected);
    }
    // The original repeats the base inertial's fluid-absence observation here.
    assert!(base.inertial.fluid_added_mass.is_none());
    let lower = model.links.get(2).unwrap();
    pose(lower.pose.transform, [0.25, 1.0, 2.1, -2., 0., 0.]);
    assert_eq!(lower.pose.relative_to, "");
}

#[test]
fn original_joint_double_pendulum() {
    let sdf = load_str(
        include_str!("fixtures/double_pendulum.sdf"),
        "double_pendulum.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().unwrap();
    assert_eq!(model.joints.len(), 2);
    let missing = model.joints.iter().find(|j| j.name == "invalid_joint");
    assert!(missing.is_none());
    let upper = model
        .joints
        .iter()
        .find(|j| j.name == "upper_joint")
        .unwrap();
    let lower = model
        .joints
        .iter()
        .find(|j| j.name == "lower_joint")
        .unwrap();
    assert_eq!(upper.parent, "base");
    assert_eq!(upper.child, "upper_link");
    assert_eq!(lower.parent, "upper_link");
    assert_eq!(lower.child, "lower_link");
    assert!(upper.pose.relative_to.is_empty());
    assert!(lower.pose.relative_to.is_empty());
    assert!(upper.axis(1).is_none());
    assert!(upper.axis(2).is_none());
    assert!(lower.axis(1).is_none());
    assert!(lower.axis(2).is_none());
    let upper_axis = upper.axis(0).unwrap();
    let lower_axis = lower.axis(0).unwrap();
    for axis in [upper_axis, lower_axis] {
        for (actual, expected) in axis.xyz.into_iter().zip([1., 0., 0.]) {
            assert!((actual - expected).abs() <= 1e-3);
        }
    }
}
