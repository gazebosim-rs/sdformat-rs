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
use sdformat::{NoResources, Sdf, Transform, World, load_str};

fn pose(actual: Transform, expected: [f64; 3]) {
    for (actual, expected) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected.into_iter().chain([0., 0., 0., 1.]))
    {
        assert!(
            (actual - expected).abs() <= 1e-3,
            "{actual} versus {expected}"
        );
    }
}
fn endpoint(sdf: &Sdf, world: &World, name: &str) -> String {
    let frame = sdf.resolve_name(&world.name, name).unwrap();
    let resolved = sdf.attached_body(&frame);
    assert!(resolved.is_ok());
    // Native world-joint endpoint names are relative to their owning world.
    resolved
        .unwrap()
        .strip_prefix(&format!("{}::", world.name))
        .unwrap()
        .into()
}
fn resolved(sdf: &Sdf, world: &World, name: &str, target: &str, expected: [f64; 3]) {
    let frame = sdf.resolve_name(&world.name, name).unwrap();
    let target = sdf.resolve_name(&world.name, target).unwrap();
    let result = sdf.pose_of(&frame, &target);
    assert!(result.is_ok());
    pose(result.unwrap(), expected);
}
#[test]
fn original_dom_joint_load_world_joint_child_frame() {
    let result = load_str(
        include_str!("fixtures/world_joint_child_frame.sdf"),
        "world_joint_child_frame.sdf",
        &NoResources,
    );
    assert!(result.is_ok(), "{:?}", result.as_ref().err());
    let sdf = result.unwrap();
    let world = sdf.worlds.first().expect("WorldByIndex(0) is present");
    assert_eq!(world.name, "world_joint_child_frame");
    assert_eq!(world.models.len(), 2);
    let first_model = world.models.first();
    assert!(first_model.is_some());
    assert!(world.models.get(1).is_some());
    assert!(world.models.get(2).is_none());
    let model = |name| world.models.iter().find(|model| model.name == name);
    assert!(model("parent_model").is_some());
    assert!(model("child_model").is_some());
    assert!(model("parent_model").unwrap().pose.relative_to.is_empty());
    assert!(model("child_model").unwrap().pose.relative_to.is_empty());
    pose(model("parent_model").unwrap().pose.transform, [0., 0., 1.]);
    pose(model("child_model").unwrap().pose.transform, [0., 0., 10.]);

    assert_eq!(world.joints.len(), 2);
    let first_joint = world.joints.first();
    assert!(first_joint.is_some());
    assert!(world.joints.get(1).is_some());
    assert!(world.joints.get(2).is_none());
    let joint = |name| world.joints.iter().find(|joint| joint.name == name);
    assert!(joint("J1").is_some());
    assert!(joint("J2").is_some());
    assert_eq!(joint("J1").unwrap().child, "child_frame");
    assert_eq!(joint("J1").unwrap().parent, "parent_model::L");
    assert_eq!(joint("J2").unwrap().child, "child_model::L");
    assert_eq!(joint("J2").unwrap().parent, "parent_model::L");
    assert_eq!(
        endpoint(&sdf, world, &joint("J1").unwrap().child),
        "child_model::L"
    );
    assert_eq!(
        endpoint(&sdf, world, &joint("J1").unwrap().parent),
        "parent_model::L"
    );
    assert_eq!(
        endpoint(&sdf, world, &joint("J2").unwrap().child),
        "child_model::L"
    );
    assert_eq!(
        endpoint(&sdf, world, &joint("J2").unwrap().parent),
        "parent_model::L"
    );
    assert!(joint("J1").unwrap().pose.relative_to.is_empty());
    assert!(joint("J2").unwrap().pose.relative_to.is_empty());
    pose(joint("J1").unwrap().pose.transform, [1., 0., 0.]);
    pose(joint("J2").unwrap().pose.transform, [10., 0., 0.]);

    assert_eq!(world.frames.len(), 1);
    let first_frame = world.frames.first();
    assert!(first_frame.is_some());
    assert!(world.frames.get(1).is_none());
    let frame = world
        .frames
        .iter()
        .find(|frame| frame.name == "child_frame");
    assert!(frame.is_some());
    pose(frame.unwrap().pose.transform, [0., 1., 0.]);
    for (name, target, expected) in [
        ("parent_model", "world", [0., 0., 1.]),
        ("child_model", "world", [0., 0., 10.]),
        ("J1", "world", [1., 1., 10.]),
        ("J2", "world", [10., 0., 10.]),
        ("child_frame", "world", [0., 1., 10.]),
        ("J1", "child_frame", [1., 0., 0.]),
        ("J1", "parent_model::L", [1., 1., 9.]),
        ("J2", "child_model::L", [10., 0., 0.]),
        ("J2", "parent_model::L", [10., 0., 9.]),
    ] {
        resolved(&sdf, world, name, target, expected);
    }
}
