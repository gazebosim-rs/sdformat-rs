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
// Original joint_dom.cc LoadJointParentWorld / ParentModelFrame / ChildModelFrame.
use sdformat::{Diagnostic, Model, NoResources, Sdf, Transform, load_str};
fn pose(actual: Transform, translation: [f64; 3]) {
    for (actual, expected) in actual.translation.into_iter().zip(translation) {
        assert!((actual - expected).abs() <= 1e-3);
    }
    for (actual, expected) in actual.rotation.into_iter().zip([0., 0., 0., 1.]) {
        assert!((actual - expected).abs() <= 1e-3);
    }
}
// Native returns body names relative to the joint's model, or the world sentinel.
fn body(sdf: &Sdf, model: &Model, name: &str) -> Result<String, Diagnostic> {
    let frame = sdf.resolve_name(&model.scoped_name, name)?;
    let body = sdf.attached_body(&frame)?;
    Ok(if body == "::" {
        "world"
    } else {
        body.strip_prefix(&format!("{}::", model.scoped_name))
            .unwrap_or(body)
    }
    .into())
}
#[test]
fn original_joint_parent_world() {
    let result = load_str(
        include_str!("fixtures/joint_parent_world.sdf"),
        "joint_parent_world.sdf",
        &NoResources,
    );
    assert!(result.is_ok());
    let sdf = result.unwrap();
    let model = sdf.models.first();
    assert!(model.is_some());
    let model = model.unwrap();
    assert_eq!(model.name, "joint_parent_world");
    assert_eq!(model.links.len(), 1);
    let indexed_link = model.links.first();
    assert!(indexed_link.is_some());
    assert!(model.links.get(1).is_none());
    pose(model.pose.transform, [1., 0., 0.]);
    assert_eq!(model.pose.relative_to, "");
    assert!(model.link_by_name("link").is_some());
    assert!(
        model
            .link_by_name("link")
            .unwrap()
            .pose
            .relative_to
            .is_empty()
    );
    pose(
        model.link_by_name("link").unwrap().pose.transform,
        [0., 0., 1.],
    );
    assert!(model.canonical_link.is_empty());
    assert_eq!(model.joints.len(), 1);
    let indexed_joint = model.joints.first();
    assert!(indexed_joint.is_some());
    assert!(model.joints.get(1).is_none());
    assert!(model.joint_by_name("joint").is_some());
    let joint = model.joint_by_name("joint").unwrap();
    assert_eq!(joint.child, "link");
    assert_eq!(joint.parent, "world");
    let resolved = body(&sdf, model, &joint.child);
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap(), "link");
    let resolved = body(&sdf, model, &joint.parent);
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap(), "world");
    pose(joint.pose.transform, [0., 0., 3.]);
    assert!(joint.pose.relative_to.is_empty());
    assert_eq!(model.frames.len(), 0);
    let indexed_frame = model.frames.first();
    assert!(indexed_frame.is_none());
}
fn model_frame_case(xml: &str, parent: bool) {
    let name = if parent {
        "joint_parent_model_frame"
    } else {
        "joint_child_model_frame"
    };
    let result = load_str(xml, name, &NoResources);
    assert!(result.is_ok());
    let sdf = result.unwrap();
    let model = sdf.models.first();
    assert!(model.is_some());
    let model = model.unwrap();
    assert_eq!(model.name, name);
    assert_eq!(model.links.len(), 2);
    assert_eq!(model.joints.len(), 1);
    let joint = model.joint_by_name("joint");
    assert!(joint.is_some());
    let joint = joint.unwrap();
    assert_eq!(joint.child, if parent { "child_link" } else { "__model__" });
    assert_eq!(
        joint.parent,
        if parent { "__model__" } else { "parent_link" }
    );
    let resolved = body(
        &sdf,
        model,
        if parent { &joint.parent } else { &joint.child },
    );
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap(), "base_link");
    let resolved = sdf.pose_of(
        &format!("{}::{}", model.scoped_name, joint.name),
        &model.scoped_name,
    );
    assert!(resolved.is_ok());
    pose(
        resolved.unwrap(),
        if parent { [1., 0., 0.] } else { [0.; 3] },
    );
}
#[test]
fn original_joint_parent_model_frame() {
    model_frame_case(include_str!("fixtures/joint_parent_model_frame.sdf"), true);
}
#[test]
fn original_joint_child_model_frame() {
    model_frame_case(include_str!("fixtures/joint_child_model_frame.sdf"), false);
}
