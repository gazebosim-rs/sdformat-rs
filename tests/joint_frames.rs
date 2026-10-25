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
// Complete original joint_dom.cc frame, relative-pose and nested endpoint cases.
use sdformat::{Diagnostic, Model, NoResources, Sdf, Transform, load_str};
fn pose(actual: Transform, translation: [f64; 3], rpy: [f64; 3]) {
    let quaternion =
        glam::DQuat::from_euler(glam::EulerRot::ZYX, rpy[2], rpy[1], rpy[0]).to_array();
    for (actual, expected) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(translation.into_iter().chain(quaternion))
    {
        assert!(
            (actual - expected).abs() <= 1e-3,
            "{actual} versus {expected}"
        );
    }
}
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
fn resolved(sdf: &Sdf, model: &Model, name: &str, target: &str, xyz: [f64; 3], rpy: [f64; 3]) {
    let origin = format!("{}::{name}", model.scoped_name);
    let target = if target == "__model__" {
        model.scoped_name.clone()
    } else {
        format!("{}::{target}", model.scoped_name)
    };
    let result = sdf.pose_of(&origin, &target);
    assert!(result.is_ok());
    pose(result.unwrap(), xyz, rpy);
}
fn frame_case(xml: &str, parent: bool) {
    let result = load_str(xml, "joint-frame.sdf", &NoResources);
    assert!(result.is_ok(), "{:?}", result.as_ref().err());
    let sdf = result.unwrap();
    let model = sdf.models.first();
    assert!(model.is_some());
    let model = model.unwrap();
    let name = if parent {
        "joint_parent_frame"
    } else {
        "joint_child_frame"
    };
    assert_eq!(model.name, name);
    assert_eq!(model.links.len(), 2);
    let indexed = model.links.first();
    assert!(indexed.is_some());
    assert!(model.links.get(1).is_some());
    assert!(model.links.get(2).is_none());
    pose(model.pose.transform, [0.; 3], [0.; 3]);
    assert_eq!(model.pose.relative_to, "");
    assert!(model.link_by_name("parent_link").is_some());
    assert!(model.link_by_name("child_link").is_some());
    assert!(
        model
            .link_by_name("parent_link")
            .unwrap()
            .pose
            .relative_to
            .is_empty()
    );
    assert!(
        model
            .link_by_name("child_link")
            .unwrap()
            .pose
            .relative_to
            .is_empty()
    );
    pose(
        model.link_by_name("parent_link").unwrap().pose.transform,
        [0., 0., 1.],
        [0.; 3],
    );
    pose(
        model.link_by_name("child_link").unwrap().pose.transform,
        [0., 0., 10.],
        [0.; 3],
    );
    assert!(model.canonical_link.is_empty());
    assert_eq!(model.joints.len(), 1);
    let indexed = model.joints.first();
    assert!(indexed.is_some());
    assert!(model.joints.get(1).is_none());
    assert!(model.joint_by_name("joint").is_some());
    let joint = model.joint_by_name("joint").unwrap();
    assert_eq!(
        joint.child,
        if parent { "child_link" } else { "child_frame" }
    );
    assert_eq!(
        joint.parent,
        if parent {
            "parent_frame"
        } else {
            "parent_link"
        }
    );
    let result = body(&sdf, model, &joint.child);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "child_link");
    let result = body(&sdf, model, &joint.parent);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "parent_link");
    assert!(joint.pose.relative_to.is_empty());
    pose(joint.pose.transform, [0., 1., 0.], [0.; 3]);
    assert_eq!(model.frames.len(), 1);
    let indexed = model.frames.first();
    assert!(indexed.is_some());
    assert!(model.frames.get(1).is_none());
    let frame = if parent {
        "parent_frame"
    } else {
        "child_frame"
    };
    assert!(model.frame_by_name(frame).is_some());
    pose(
        model.frame_by_name(frame).unwrap().pose.transform,
        [1., 0., 0.],
        [0.; 3],
    );
    resolved(
        &sdf,
        model,
        "parent_link",
        "__model__",
        [0., 0., 1.],
        [0.; 3],
    );
    resolved(
        &sdf,
        model,
        "child_link",
        "__model__",
        [0., 0., 10.],
        [0.; 3],
    );
    let x = if parent { 0. } else { 1. };
    resolved(&sdf, model, "joint", "__model__", [x, 1., 10.], [0.; 3]);
    resolved(
        &sdf,
        model,
        frame,
        "__model__",
        [1., 0., if parent { 1. } else { 10. }],
        [0.; 3],
    );
    resolved(&sdf, model, "joint", "child_link", [x, 1., 0.], [0.; 3]);
    resolved(&sdf, model, "joint", "parent_link", [x, 1., 9.], [0.; 3]);
}
#[test]
fn original_joint_parent_frame() {
    frame_case(include_str!("fixtures/joint_parent_frame.sdf"), true);
}
#[test]
fn original_joint_child_frame() {
    frame_case(include_str!("fixtures/joint_child_frame.sdf"), false);
}
#[test]
fn original_joint_pose_relative_to() {
    let result = load_str(
        include_str!("fixtures/model_joint_relative_to.sdf"),
        "model_joint_relative_to.sdf",
        &NoResources,
    );
    assert!(result.is_ok(), "{:?}", result.as_ref().err());
    let sdf = result.unwrap();
    let model = sdf.models.first();
    assert!(model.is_some());
    let model = model.unwrap();
    assert_eq!(model.name, "model_joint_relative_to");
    assert_eq!(model.links.len(), 4);
    let indexed = model.links.first();
    assert!(indexed.is_some());
    for index in 1..4 {
        assert!(model.links.get(index).is_some());
    }
    assert!(model.links.get(4).is_none());
    pose(model.pose.transform, [0.; 3], [0.; 3]);
    assert_eq!(model.pose.relative_to, "");
    for name in ["P1", "P2", "C1", "C2"] {
        assert!(model.link_by_name(name).is_some());
    }
    for name in ["P1", "P2", "C1"] {
        assert!(
            model
                .link_by_name(name)
                .unwrap()
                .pose
                .relative_to
                .is_empty()
        );
    }
    assert_eq!(model.link_by_name("C2").unwrap().pose.relative_to, "J2");
    let h = std::f64::consts::FRAC_PI_2;
    for (name, xyz, rpy) in [
        ("P1", [1., 0., 0.], [0., h, 0.]),
        ("C1", [2., 0., 0.], [0., -h, 0.]),
        ("P2", [3., 0., 0.], [0., h, 0.]),
        ("C2", [4., 0., 0.], [0.; 3]),
    ] {
        pose(model.link_by_name(name).unwrap().pose.transform, xyz, rpy);
    }
    assert!(model.canonical_link.is_empty());
    assert_eq!(model.joints.len(), 2);
    let indexed = model.joints.first();
    assert!(indexed.is_some());
    assert!(model.joints.get(1).is_some());
    assert!(model.joints.get(2).is_none());
    assert!(model.joint_by_name("J1").is_some());
    assert!(model.joint_by_name("J2").is_some());
    assert!(
        model
            .joint_by_name("J1")
            .unwrap()
            .pose
            .relative_to
            .is_empty()
    );
    assert_eq!(model.joint_by_name("J2").unwrap().pose.relative_to, "P2");
    pose(
        model.joint_by_name("J1").unwrap().pose.transform,
        [0., 0., 1.],
        [0.; 3],
    );
    pose(
        model.joint_by_name("J2").unwrap().pose.transform,
        [0., 0., 2.],
        [0.; 3],
    );
    for (name, xyz, rpy) in [
        ("P1", [1., 0., 0.], [0., h, 0.]),
        ("C1", [2., 0., 0.], [0., -h, 0.]),
        ("J1", [1., 0., 0.], [0., -h, 0.]),
        ("P2", [3., 0., 0.], [0., h, 0.]),
        ("J2", [5., 0., 0.], [0., h, 0.]),
        ("C2", [5., 0., -4.], [0., h, 0.]),
    ] {
        resolved(&sdf, model, name, "__model__", xyz, rpy);
    }
    resolved(&sdf, model, "J1", "C1", [0., 0., 1.], [0.; 3]);
    resolved(&sdf, model, "J2", "P2", [0., 0., 2.], [0.; 3]);
    assert_eq!(model.frames.len(), 0);
    let indexed = model.frames.first();
    assert!(indexed.is_none());
}
#[test]
fn original_joint_nested_parent_child() {
    let result = load_str(
        include_str!("fixtures/joint_nested_parent_child.sdf"),
        "joint_nested_parent_child.sdf",
        &NoResources,
    );
    assert!(result.is_ok(), "{:?}", result.as_ref().err());
    let sdf = result.unwrap();
    let model = sdf.models.first();
    assert!(model.is_some());
    let model = model.unwrap();
    let h = std::f64::consts::FRAC_PI_2;
    for (name, parent, child, parent_body, child_body, xyz, rpy) in [
        (
            "J1",
            "M1::L1",
            "L1",
            "M1::L1",
            "L1",
            [0., 0., 9.],
            [0., h, 0.],
        ),
        ("J2", "F1", "L1", "M1::L1", "L1", [0., 1., 10.], [0., h, 0.]),
        ("J3", "L1", "M1::L2", "L1", "M1::L2", [1., 1., 1.], [0.; 3]),
        ("J4", "L1", "M1::F1", "L1", "M1::L1", [1., 0., 1.], [0.; 3]),
        (
            "J5",
            "L1",
            "M1::M2",
            "L1",
            "M1::M2::L1",
            [0., -1., 1.],
            [h, 0., 0.],
        ),
        (
            "J6",
            "M1::__model__",
            "L1",
            "M1::L1",
            "L1",
            [1., 0., 10.],
            [0., h, 0.],
        ),
        (
            "J7",
            "L1",
            "M1::__model__",
            "L1",
            "M1::L1",
            [0., 0., 1.],
            [0.; 3],
        ),
    ] {
        let joint = model.joint_by_name(name);
        assert!(joint.is_some());
        let joint = joint.unwrap();
        assert_eq!(joint.parent, parent);
        assert_eq!(joint.child, child);
        let result = body(&sdf, model, &joint.parent);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), parent_body);
        let result = body(&sdf, model, &joint.child);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), child_body);
        resolved(&sdf, model, name, "__model__", xyz, rpy);
    }
}

#[test]
fn actual_native_nested_implicit_model_reference_corpus() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("model_frame_alias.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 150);
    for row in cases.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let tokens = row["tokens"].as_array().unwrap();
        let parsed = load_str(
            row["xml"].as_str().unwrap(),
            "model-frame.sdf",
            &NoResources,
        );
        if row.get("gap").is_some() {
            assert_eq!(id, "static_joint_child");
            assert_eq!(tokens[0], "0");
            // Historical gap metadata is retained byte-for-byte. This exact
            // native-positive payload now follows the ordinary assertions below.
            assert!(parsed.is_ok(), "{:?}", parsed.as_ref().err());
        }
        assert_eq!(
            parsed.is_ok(),
            tokens[0] == "0",
            "{id}: {:?}",
            parsed.as_ref().err()
        );
        if tokens[0] != "0" {
            continue;
        }
        let sdf = parsed.unwrap();
        let model = sdf
            .models
            .first()
            .unwrap_or_else(|| &sdf.worlds[0].models[0]);
        let kind = row["kind"].as_str().unwrap();
        let result = match kind {
            "model" => sdf.pose_of(&model.scoped_name, "::"),
            "collision" => {
                let link = model.link_by_name("probe").unwrap();
                sdf.resolve_pose(
                    &link.collisions[0].pose,
                    &model.scoped_name,
                    &link.scoped_name,
                    &model.scoped_name,
                )
            }
            _ => sdf.pose_of(&format!("{}::probe", model.scoped_name), &model.scoped_name),
        };
        let mut expected = tokens.iter().skip(1).map(|v| v.as_str().unwrap());
        let errors = expected.next().unwrap().parse::<usize>().unwrap();
        if errors == 0 {
            let actual = result.unwrap();
            for component in actual.translation.into_iter().chain(actual.rotation) {
                let native = f64::from_bits(expected.next().unwrap().parse().unwrap());
                assert!(
                    (component - native).abs() <= 1e-12,
                    "{id}: {component} versus {native}"
                );
            }
        } else {
            assert_eq!(
                result.unwrap_err().code,
                sdformat::ErrorCode::UnknownReference,
                "{id}"
            );
            for _ in 0..errors {
                assert_eq!(expected.next(), Some("26"), "{id}");
            }
        }
        if kind == "joint" {
            let axis = model.joint_by_name("probe").unwrap().axis.as_ref();
            assert_eq!(axis.is_some(), expected.next().unwrap() == "1", "{id}");
            if let Some(axis) = axis {
                assert_eq!(expected.next(), Some("0"), "{id}");
                let actual = sdf
                    .resolve_axis(
                        axis,
                        &format!("{}::probe", model.scoped_name),
                        &model.scoped_name,
                    )
                    .unwrap();
                for component in actual {
                    let native = f64::from_bits(expected.next().unwrap().parse().unwrap());
                    assert!((component - native).abs() <= 1e-12, "{id}");
                }
            }
        }
        assert!(expected.next().is_none(), "{id}");
        for (reference, name) in [("n::__model__", "n"), ("n::d::__model__", "n::d")] {
            assert_eq!(
                sdf.resolve_name(&model.scoped_name, reference).unwrap(),
                format!("{}::{name}", model.scoped_name),
                "{id}"
            );
        }
        assert!(
            sdf.resolve_name(&model.scoped_name, "l::__model__")
                .is_err(),
            "{id}"
        );
    }
}
