// Copyright (C) 2026 Open Source Robotics Foundation
// Copyright 2015 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
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
// Ports: sdformat/test/integration/nested_model.cc,
// NestedModel.NestedModelWithFrames / NestedModelWithSiblingFrames.
use glam::{DQuat, DVec3};
use sdformat::*;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

struct Frames;
impl ResourceResolver for Frames {
    fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
        assert_eq!(uri, "test_model_with_frames");
        Ok(Resource {
            canonical_uri: "fixture:test_model_with_frames".into(),
            xml: include_str!("fixtures/test_model_with_frames.sdf").into(),
        })
    }
}
fn pose(translation: [f64; 3], rotation: DQuat) -> Transform {
    Transform {
        translation,
        rotation: rotation.to_array(),
    }
}
fn compose(a: Transform, b: Transform) -> Transform {
    let q = DQuat::from_array(a.rotation);
    pose(
        (DVec3::from_array(a.translation) + q * DVec3::from_array(b.translation)).to_array(),
        q * DQuat::from_array(b.rotation),
    )
}
fn inverse(a: Transform) -> Transform {
    let q = DQuat::from_array(a.rotation).conjugate();
    pose((q * -DVec3::from_array(a.translation)).to_array(), q)
}
fn close(actual: Transform, expected: Transform) {
    // Pose3::operator== delegates to component-wise Vector3/Quaternion equality.
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected.translation.into_iter().chain(expected.rotation))
    {
        assert!(
            (a - b).abs() <= 1e-3,
            "original equality: {actual:?} vs {expected:?}"
        );
        assert!(
            (a - b).abs() < 1e-6,
            "supplementary tighter bound: {actual:?} vs {expected:?}"
        );
    }
}
fn model_pose() -> Transform {
    pose([10., 0., 0.], DQuat::from_rotation_z(FRAC_PI_2))
}
fn document(sibling: bool) -> Sdf {
    let frames = if sibling {
        "<frame name='parentModelFrame'/><frame name='testFrame'><pose>0 5 0 0 0 -1.5707963267948966</pose></frame>"
    } else {
        ""
    };
    let relative = if sibling {
        " relative_to='testFrame'"
    } else {
        ""
    };
    let xml = format!(
        "<sdf version='1.7'><world name='default'><model name='ParentModel'>{frames}<include><uri>test_model_with_frames</uri><name>M1</name><pose{relative}>10 0 0 0 0 1.5707963267948966</pose></include></model></world></sdf>"
    );
    load_str(&xml, "nested-frames", &Frames).unwrap()
}
#[test]
fn nested_model_with_frames_original_assertions() {
    let sdf = document(false);
    let world = &sdf.worlds[0];
    let parent = &world.models[0];
    let child = &parent.models[0];
    let frame1 = compose(
        model_pose(),
        pose([0.; 3], DQuat::from_rotation_x(FRAC_PI_2)),
    );
    let frame2 = compose(frame1, pose([0.; 3], DQuat::from_rotation_y(FRAC_PI_4)));
    for (name, expected) in [("F1", frame1), ("F2", frame2)] {
        let resolved = sdf
            .pose_of(
                &parent.scoped_name,
                &format!("{}::{name}", child.scoped_name),
            )
            .unwrap();
        close(inverse(resolved), expected);
    }
    let link1 = child.links.iter().find(|l| l.name == "L1").unwrap();
    close(
        inverse(
            sdf.pose_of(&parent.scoped_name, &link1.scoped_name)
                .unwrap(),
        ),
        frame1,
    );
    let visual = link1.visuals.iter().find(|v| v.name == "V1").unwrap();
    assert_eq!(visual.pose.relative_to, "F2");
    let collision = link1.collisions.iter().find(|v| v.name == "C1").unwrap();
    assert_eq!(collision.pose.relative_to, "__model__");
    let link2 = child.links.iter().find(|l| l.name == "L2").unwrap();
    close(
        inverse(
            sdf.pose_of(&parent.scoped_name, &link2.scoped_name)
                .unwrap(),
        ),
        compose(frame1, pose([1., 0., 0.], DQuat::IDENTITY)),
    );
    let joint = child.joints.iter().find(|j| j.name == "J1").unwrap();
    let joint_name = format!("{}::{}", child.scoped_name, joint.name);
    close(
        inverse(sdf.pose_of(&parent.scoped_name, &joint_name).unwrap()),
        frame1,
    );
    for (axis, unit) in [
        (joint.axis.as_ref().unwrap(), DVec3::Z),
        (joint.axis2.as_ref().unwrap(), DVec3::X),
    ] {
        let actual = DQuat::from_array(model_pose().rotation)
            * DVec3::from_array(
                sdf.resolve_axis(axis, &joint_name, &child.scoped_name)
                    .unwrap(),
            );
        let expected = DQuat::from_array(frame2.rotation) * unit;
        for (a, b) in actual.to_array().into_iter().zip(expected.to_array()) {
            assert!(
                (a - b).abs() <= 1e-3,
                "original vector equality: {actual:?} vs {expected:?}"
            );
        }
        assert!(
            actual.abs_diff_eq(expected, 1e-6),
            "{actual:?} vs {expected:?}"
        );
    }
}
#[test]
fn nested_model_with_sibling_frames_original_assertions() {
    let sdf = document(true);
    let world = &sdf.worlds[0];
    let parent = &world.models[0];
    assert_eq!(parent.frames[0].name, "parentModelFrame");
    let child = parent.models.iter().find(|m| m.name == "M1").unwrap();
    assert_eq!(child.pose.relative_to, "testFrame");
    let nested = compose(
        pose([0., 5., 0.], DQuat::from_rotation_z(-FRAC_PI_2)),
        model_pose(),
    );
    close(
        sdf.pose_of(&child.scoped_name, &parent.scoped_name)
            .unwrap(),
        nested,
    );
    let frame1 = compose(nested, pose([0.; 3], DQuat::from_rotation_x(FRAC_PI_2)));
    let frame2 = compose(frame1, pose([0.; 3], DQuat::from_rotation_y(FRAC_PI_4)));
    for (name, expected) in [("F1", frame1), ("F2", frame2)] {
        child.frames.iter().find(|f| f.name == name).unwrap();
        close(
            inverse(
                sdf.pose_of(
                    &parent.scoped_name,
                    &format!("{}::{name}", child.scoped_name),
                )
                .unwrap(),
            ),
            expected,
        );
    }
    for (name, expected) in [
        ("L1", frame1),
        ("L2", compose(frame1, pose([1., 0., 0.], DQuat::IDENTITY))),
    ] {
        let link = child.links.iter().find(|l| l.name == name).unwrap();
        close(
            inverse(sdf.pose_of(&parent.scoped_name, &link.scoped_name).unwrap()),
            expected,
        );
    }
    let joint = child.joints.iter().find(|j| j.name == "J1").unwrap();
    close(
        inverse(
            sdf.pose_of(
                &parent.scoped_name,
                &format!("{}::{}", child.scoped_name, joint.name),
            )
            .unwrap(),
        ),
        frame1,
    );
}
