// Copyright (C) 2026 Open Source Robotics Foundation
// Copyright 2018 Open Source Robotics Foundation
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
// Ports: sdformat/test/integration/{joint_dom,world_dom}.cc,
// DOMJoint.LoadLinkJointSameName16Valid and DOMWorld.LoadModelFrameSameName.
use sdformat::*;

fn original_pose(actual: Transform, xyz: [f64; 3]) {
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(xyz.into_iter().chain([0., 0., 0., 1.]))
    {
        assert!(
            (a - b).abs() <= 1e-3,
            "original Pose3 comparison: {actual:?}"
        );
        assert!(
            (a - b).abs() < 1e-6,
            "supplementary tighter comparison: {actual:?}"
        );
    }
}
#[test]
fn load_link_joint_same_name16_valid_original_assertions() {
    let sdf = load_str(
        include_str!("fixtures/model_link_joint_same_name.sdf"),
        "model_link_joint_same_name.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().unwrap();
    assert_eq!(model.name, "link_joint_same_name");
    assert_eq!(model.links.len(), 2);
    assert!(!model.links.is_empty()); // Original LinkByIndex(0) presence.
    assert!(model.links.get(1).is_some());
    assert!(model.links.get(2).is_none());
    original_pose(model.pose.transform, [0., 0., 0.]);
    assert_eq!(model.pose.relative_to, "");
    let base = model.links.iter().find(|link| link.name == "base").unwrap();
    let attachment = model
        .links
        .iter()
        .find(|link| link.name == "attachment")
        .unwrap();
    assert!(base.pose.relative_to.is_empty());
    assert!(attachment.pose.relative_to.is_empty());
    original_pose(base.pose.transform, [1., 0., 0.]);
    original_pose(attachment.pose.transform, [0., 2., 0.]);
    assert!(model.canonical_link.is_empty());
    assert_eq!(model.joints.len(), 1);
    assert!(!model.joints.is_empty()); // Original JointByIndex(0) presence.
    assert!(model.joints.get(1).is_none());
    assert!(!model.joints.iter().any(|joint| joint.name == "attachment"));
    let joint = model
        .joints
        .iter()
        .find(|joint| joint.name == "attachment_joint")
        .unwrap();
    assert!(joint.pose.relative_to.is_empty());
    original_pose(joint.pose.transform, [0., 0., 3.]);
    let joint_name = format!("{}::{}", model.scoped_name, joint.name);
    original_pose(
        sdf.pose_of(&base.scoped_name, &model.scoped_name).unwrap(),
        [1., 0., 0.],
    );
    original_pose(
        sdf.pose_of(&attachment.scoped_name, &model.scoped_name)
            .unwrap(),
        [0., 2., 0.],
    );
    original_pose(
        sdf.pose_of(&joint_name, &model.scoped_name).unwrap(),
        [0., 2., 3.],
    );
    original_pose(
        sdf.pose_of(&attachment.scoped_name, &base.scoped_name)
            .unwrap(),
        [-1., 2., 0.],
    );
    original_pose(
        sdf.pose_of(&joint_name, &base.scoped_name).unwrap(),
        [-1., 2., 3.],
    );
    original_pose(
        sdf.pose_of(&joint_name, &attachment.scoped_name).unwrap(),
        [0., 0., 3.],
    );
}

#[test]
fn load_world_model_frame_same_name_original_assertions() {
    let sdf = load_str(
        include_str!("fixtures/world_model_frame_same_name_1_6.sdf"),
        "world_model_frame_same_name_1_6.sdf",
        &NoResources,
    )
    .unwrap();
    let world = sdf.worlds.first().unwrap();
    assert_eq!(world.name, "world_model_frame_same_name");
    assert_eq!(world.models.len(), 2);
    assert!(!world.models.is_empty()); // Original ModelByIndex(0) presence.
    assert!(world.models.get(1).is_some());
    assert!(world.models.get(2).is_none());
    let base = world
        .models
        .iter()
        .find(|model| model.name == "base")
        .unwrap();
    let ground = world
        .models
        .iter()
        .find(|model| model.name == "ground")
        .unwrap();
    assert!(base.pose.relative_to.is_empty());
    assert!(ground.pose.relative_to.is_empty());
    original_pose(base.pose.transform, [1., 0., 0.]);
    original_pose(ground.pose.transform, [0., 2., 0.]);
    assert_eq!(world.frames.len(), 1);
    assert!(!world.frames.is_empty()); // Original FrameByIndex(0) presence.
    assert!(world.frames.get(1).is_none());
    assert!(!world.frames.iter().any(|frame| frame.name == "ground"));
    let frame = world
        .frames
        .iter()
        .find(|frame| frame.name == "ground_frame")
        .unwrap();
    assert!(frame.pose.relative_to.is_empty());
    original_pose(frame.pose.transform, [0., 0., 3.]);
    let frame_name = format!("{}::{}", world.name, frame.name);
    original_pose(
        sdf.pose_of(&base.scoped_name, &world.name).unwrap(),
        [1., 0., 0.],
    );
    original_pose(
        sdf.pose_of(&ground.scoped_name, &world.name).unwrap(),
        [0., 2., 0.],
    );
    original_pose(sdf.pose_of(&frame_name, &world.name).unwrap(), [0., 0., 3.]);
    original_pose(
        sdf.pose_of(&ground.scoped_name, &base.scoped_name).unwrap(),
        [-1., 2., 0.],
    );
    original_pose(
        sdf.pose_of(&frame_name, &base.scoped_name).unwrap(),
        [-1., 0., 3.],
    );
    original_pose(
        sdf.pose_of(&frame_name, &ground.scoped_name).unwrap(),
        [0., -2., 3.],
    );
}
