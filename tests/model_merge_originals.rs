/*
 * Copyright 2019 Open Source Robotics Foundation
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
// Fixed observable predicates from IncludesTest.MergeInclude and PlacementFrame.
use glam::{DQuat, DVec3};
use sdformat::*;

struct Fixtures;
impl ResourceResolver for Fixtures {
    fn resolve(&self, uri: &str, _parent: &str) -> Result<Resource, String> {
        if uri != "merge_robot" {
            return Err(format!("unknown fixture {uri}"));
        }
        Ok(Resource {
            canonical_uri: "resolved://merge_robot/model.sdf".into(),
            xml: include_str!("fixtures/merge_robot_original.sdf").into(),
        })
    }
}
fn loaded() -> Sdf {
    load_str(
        include_str!("fixtures/merge_include_model_original.sdf"),
        "merge_include_model.sdf",
        &Fixtures,
    )
    .expect("native errors.empty()")
}
fn transform(xyz: [f64; 3], q: DQuat) -> Transform {
    Transform {
        translation: xyz,
        rotation: q.to_array(),
    }
}
fn compose(a: Transform, b: Transform) -> Transform {
    let q = DQuat::from_array(a.rotation);
    transform(
        (DVec3::from_array(a.translation) + q * DVec3::from_array(b.translation)).to_array(),
        q * DQuat::from_array(b.rotation),
    )
}
fn inverse(value: Transform) -> Transform {
    let q = DQuat::from_array(value.rotation).conjugate();
    transform((q * -DVec3::from_array(value.translation)).to_array(), q)
}
fn native_pose_eq(actual: Transform, expected: Transform) {
    // Pinned Pose3 operator== compares each Vector3/Quaternion component at1e-3.
    // Quaternion signs are compared directly, without sign equivalence.
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected.translation.into_iter().chain(expected.rotation))
    {
        assert!((a - b).abs() <= 1e-3, "{a} != {b}");
    }
}

#[test]
fn original_includes_test_merge_include_placement_frame() {
    let sdf = loaded();
    let world = sdf.worlds.first().expect("WorldByIndex(0) != nullptr");
    let model = world
        .models
        .get(1)
        .expect("ModelByIndex(1) is dereferenced");
    assert_eq!(model.name, "robot2");
    assert_eq!(model.links.len(), 7);
    assert_eq!(model.joints.len(), 6);
    let top = model
        .link_by_name("top")
        .expect("LinkByName(top) != nullptr");
    let pose = sdf
        .pose_of(&top.scoped_name, &model.scoped_name)
        .expect("SemanticPose.Resolve errors.empty()");
    native_pose_eq(
        pose,
        Transform {
            translation: [0., 0., 2.],
            rotation: [0., 0., 0., 1.],
        },
    );
}

#[test]
fn partial_original_includes_test_merge_include() {
    let sdf = loaded();
    let world = sdf.worlds.first().expect("WorldByIndex(0) != nullptr");
    let model = world
        .models
        .first()
        .expect("ModelByIndex(0) is dereferenced");
    assert_eq!(model.name, "robot1");
    assert_eq!(model.links.len(), 7);
    assert_eq!(model.joints.len(), 6);
    assert_eq!(model.models.len(), 1);
    let canonical = model
        .canonical_link_and_relative_name()
        .0
        .expect("CanonicalLink() != nullptr");
    assert!(
        std::ptr::eq(canonical, &model.links[0]),
        "CanonicalLink aliases LinkByIndex(0)"
    );
    let imported = load_str(
        include_str!("fixtures/merge_robot_original.sdf"),
        "model.sdf",
        &NoResources,
    )
    .expect("includedModel.Load errors.empty()");
    let imported = imported
        .models
        .first()
        .expect("includedModel.Model() != nullptr");
    let proxy = format!("_merged__{}__model__", imported.name);
    let parent = transform(
        [100., 0., 0.],
        DQuat::from_rotation_x(std::f64::consts::FRAC_PI_4),
    );
    // Native oracle uses decimal -1.5707, deliberately different from -PI/2.
    // This ratio preserves that exact f64 instead of substituting a math constant.
    const NATIVE_WHEEL_ROLL: f64 = -15_707.0 / 10_000.0;
    let right_wheel = transform(
        [0.554282, -0.625029, -0.025],
        DQuat::from_rotation_x(NATIVE_WHEEL_ROLL),
    );
    let left_wheel = transform(
        [0.554282, 0.625029, -0.025],
        DQuat::from_rotation_x(NATIVE_WHEEL_ROLL),
    );
    for (name, local) in [
        (
            "chassis",
            transform([-0.151427, 0., 0.175], DQuat::IDENTITY),
        ),
        ("top", transform([0.6, 0., 0.7], DQuat::IDENTITY)),
    ] {
        let link = model.link_by_name(name).expect("named link != nullptr");
        assert_eq!(link.pose.relative_to, proxy);
        let actual = sdf
            .pose_of(&link.scoped_name, &model.scoped_name)
            .expect("Resolve errors.empty()");
        native_pose_eq(actual, compose(parent, local));
    }
    let joint = model
        .joint_by_name("right_wheel_joint")
        .expect("right_wheel_joint != nullptr");
    assert_eq!(joint.pose.relative_to, proxy);
    let joint_frame = format!("{}::{}", model.scoped_name, joint.name);
    let child = sdf.resolve_name(&model.scoped_name, &joint.child).unwrap();
    native_pose_eq(
        sdf.pose_of(&joint_frame, &child)
            .expect("Resolve errors.empty()"),
        compose(
            inverse(right_wheel),
            transform([1., 0., 0.], DQuat::IDENTITY),
        ),
    );
    let frame = model
        .frame_by_name("sensor_frame")
        .expect("sensor_frame != nullptr");
    assert_eq!(frame.pose.relative_to, proxy);
    let frame_name = format!("{}::{}", model.scoped_name, frame.name);
    native_pose_eq(
        sdf.pose_of(&frame_name, &model.scoped_name)
            .expect("Resolve errors.empty()"),
        compose(
            parent,
            transform(
                [0., 1., 0.],
                DQuat::from_rotation_y(std::f64::consts::FRAC_PI_4),
            ),
        ),
    );
    let joint = model
        .joint_by_name("left_wheel_joint")
        .expect("left_wheel_joint != nullptr");
    let axis = joint.axis.as_ref().expect("Axis(0) != nullptr");
    let frame = format!("{}::{}", model.scoped_name, joint.name);
    let xyz = sdf
        .resolve_axis(axis, &frame, &frame)
        .expect("ResolveXyz errors.empty()");
    let expected = DQuat::from_array(left_wheel.rotation).conjugate() * DVec3::Z;
    for (a, b) in xyz.into_iter().zip(expected.to_array()) {
        assert!(
            (a - b).abs() <= 1e-3,
            "native Vector3 operator== component: {a} != {b}"
        );
    }
    assert_eq!(
        model
            .joint_by_name("test_model_parent")
            .expect("joint != nullptr")
            .parent,
        proxy
    );
    assert_eq!(
        model
            .joint_by_name("test_model_child")
            .expect("joint != nullptr")
            .child,
        proxy
    );
    let plugin = model.plugins.first().expect("retained plugin metadata");
    assert_eq!(plugin.name, "test");
    let custom = model
        .extensions
        .iter()
        .find(|e| e.tag == "custom:foo")
        .expect("retained custom metadata");
    assert_eq!(custom.attribute("name"), Some("baz"));
    assert!(!model.extensions.iter().any(|e| e.tag == "unknown_element"));
    // UNPORTED: native Model.Element()/FindElement pointer API and generic
    // HasElement(enable_wind/static) absence. False DTO flags cannot stand in
    // for XML-element absence. This original remains explicitly PARTIAL.
}
