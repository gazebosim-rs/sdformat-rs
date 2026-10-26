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
// Fixed-fixture observations from DOMJointAxis.XyzExpressedIn.
use sdformat::{JointKind, NoResources, Pose, load_str};

fn close<const N: usize>(actual: [f64; N], expected: [f64; N]) {
    for (a, e) in actual.into_iter().zip(expected) {
        assert!((a - e).abs() <= 1e-15, "{a} differs from {e}");
    }
}

fn raw_pose(pose: &Pose, translation: [f64; 3], pitch: f64, relative_to: &str) {
    assert_eq!(pose.relative_to, relative_to);
    assert_eq!(pose.transform.translation, translation);
    close(
        pose.transform.rotation,
        [0., (pitch / 2.).sin(), 0., (pitch / 2.).cos()],
    );
}

#[test]
fn original_dom_joint_axis_xyz_expressed_in() {
    let sdf = load_str(
        include_str!("fixtures/model_joint_axis_expressed_in.sdf"),
        "model_joint_axis_expressed_in.sdf",
        &NoResources,
    )
    .expect("native Root.Load has no errors");
    let model = sdf.models.first().expect("native Root.Model is present");
    let scope = "model_joint_axis_expressed_in";
    assert_eq!(model.name, scope);
    assert_eq!(model.links.len(), 4);
    for index in 0..4 {
        assert!(model.links.get(index).is_some(), "LinkByIndex({index})");
    }
    assert!(model.links.get(4).is_none());
    raw_pose(&model.pose, [0.; 3], 0., "");

    for name in ["P1", "P2", "C1", "C2"] {
        assert!(model.link_by_name(name).is_some(), "LinkNameExists({name})");
    }
    let p1 = model.link_by_name("P1").unwrap();
    let p2 = model.link_by_name("P2").unwrap();
    let c1 = model.link_by_name("C1").unwrap();
    let c2 = model.link_by_name("C2").unwrap();
    raw_pose(&p1.pose, [1., 0., 0.], std::f64::consts::FRAC_PI_2, "");
    raw_pose(&p2.pose, [3., 0., 0.], std::f64::consts::FRAC_PI_2, "");
    raw_pose(&c1.pose, [2., 0., 0.], -std::f64::consts::FRAC_PI_2, "");
    raw_pose(&c2.pose, [4., 0., 0.], 0., "J2");
    assert!(model.canonical_link.is_empty());

    assert_eq!(model.joints.len(), 2);
    assert!(!model.joints.is_empty());
    assert!(model.joints.get(1).is_some());
    assert!(model.joints.get(2).is_none());
    for name in ["J1", "J2"] {
        assert!(
            model.joint_by_name(name).is_some(),
            "JointNameExists({name})"
        );
    }
    let j1 = model.joint_by_name("J1").unwrap();
    let j2 = model.joint_by_name("J2").unwrap();
    assert_eq!(j1.kind, JointKind::Revolute);
    assert_eq!(j2.kind, JointKind::Revolute);
    let a1 = j1.axis(0).expect("J1 Axis(0)");
    let a2 = j2.axis(0).expect("J2 Axis(0)");
    assert!(j1.axis(1).is_none());
    assert!(j2.axis(1).is_none());
    assert_eq!(j1.pose.relative_to, "");
    assert_eq!(j2.pose.relative_to, "P2");
    assert_eq!(j1.pose.transform.rotation, [0., 0., 0., 1.]);
    assert_eq!(j2.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(a1.expressed_in.is_empty());
    assert_eq!(a2.expressed_in, "__model__");
    assert_eq!(a1.xyz, [0., 0., 1.]);
    assert_eq!(a2.xyz, [0., 0., 1.]);

    let j1_frame = format!("{scope}::J1");
    let j2_frame = format!("{scope}::J2");
    let c1_frame = format!("{scope}::C1");
    let c2_frame = format!("{scope}::C2");
    close(
        sdf.resolve_axis(a1, &j1_frame, &c1_frame).unwrap(),
        [0., 0., 1.],
    );
    close(
        sdf.resolve_axis(a2, &j2_frame, scope).unwrap(),
        [0., 0., 1.],
    );
    // Native's omitted target is the joint child link; the Rust API names it.
    assert_eq!(j1.child, "C1");
    close(
        sdf.resolve_axis(a1, &j1_frame, &c1_frame).unwrap(),
        [0., 0., 1.],
    );

    close(
        sdf.pose_of(&j1_frame, scope).unwrap().rotation,
        [
            0.,
            (-std::f64::consts::FRAC_PI_4).sin(),
            0.,
            std::f64::consts::FRAC_PI_4.cos(),
        ],
    );
    close(
        sdf.pose_of(&j2_frame, scope).unwrap().rotation,
        [
            0.,
            std::f64::consts::FRAC_PI_4.sin(),
            0.,
            std::f64::consts::FRAC_PI_4.cos(),
        ],
    );
    close(
        sdf.resolve_axis(a1, &j1_frame, scope).unwrap(),
        [-1., 0., 0.],
    );
    close(
        sdf.resolve_axis(a2, &j2_frame, scope).unwrap(),
        [0., 0., 1.],
    );
    close(
        sdf.resolve_axis(a1, &j1_frame, &c1_frame).unwrap(),
        [0., 0., 1.],
    );
    close(
        sdf.resolve_axis(a2, &j2_frame, &c2_frame).unwrap(),
        [-1., 0., 0.],
    );
    assert_eq!(j2.child, "C2");
    close(
        sdf.resolve_axis(a2, &j2_frame, &c2_frame).unwrap(),
        [-1., 0., 0.],
    );
    assert_eq!(model.frames.len(), 0);
    assert!(model.frames.is_empty());
}
