// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[test]
fn native_joint_default_and_local_target_semantics() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("joint_semantic_pose.json")).unwrap();
    for row in rows {
        let sdf = load_str(row["xml"].as_str().unwrap(), "joint-pose.sdf", &NoResources).unwrap();
        let result = sdf.joint_pose(
            row["scoped_joint"].as_str().unwrap(),
            row["target"].as_str().unwrap(),
        );
        let tokens = row["tokens"].as_array().unwrap();
        if tokens[0] == "REJECT" {
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[1], "26");
            assert_eq!(
                result.unwrap_err().code,
                ErrorCode::UnknownReference,
                "{}",
                row["id"]
            );
        } else {
            assert_eq!(tokens[0], "OK");
            assert_eq!(tokens.len(), 8);
            let pose = result.unwrap();
            for (actual, expected) in pose
                .translation
                .into_iter()
                .chain(pose.rotation)
                .zip(tokens.iter().skip(1))
            {
                let expected = f64::from_bits(expected.as_str().unwrap().parse().unwrap());
                assert!(
                    (actual - expected).abs() <= 1e-3,
                    "{}: {actual} != {expected}",
                    row["id"]
                );
            }
        }
    }
}
#[test]
fn joint_identity_and_diagnostics_are_explicit() {
    let sdf = load_str(
        include_str!("fixtures/model_joint_relative_to.sdf"),
        "caller.sdf",
        &NoResources,
    )
    .unwrap();
    let error = sdf.joint_pose("missing", "").unwrap_err();
    assert_eq!(error.code, ErrorCode::UnknownReference);
    assert_eq!(error.source.uri, "caller.sdf");
    let error = sdf
        .joint_pose("model_joint_relative_to::P1", "")
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!(error.source.uri, "caller.sdf");
    assert!(error.source.line > 0);
    let error = sdf
        .joint_pose("model_joint_relative_to::J1", "missing")
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::UnknownReference);
    assert_eq!(
        error.source,
        sdf.models[0].joint_by_name("J1").unwrap().source
    );
    assert_eq!(
        sdf.joint_pose("model_joint_relative_to::J1", "J1").unwrap(),
        Transform::default()
    );
}
#[test]
fn declared_child_target_and_loaded_graph_survive_dom_edits() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("joint_semantic_pose.json")).unwrap();
    let row = rows
        .iter()
        .find(|row| row["id"] == "model-child-frame:default")
        .unwrap();
    let mut sdf = load_str(row["xml"].as_str().unwrap(), "model.sdf", &NoResources).unwrap();
    let before = sdf.joint_pose("w::m::j", "").unwrap();
    assert_eq!(before, sdf.joint_pose("w::m::j", "child_frame").unwrap());
    assert_ne!(before, sdf.joint_pose("w::m::j", "c").unwrap());
    let joint = &mut sdf.worlds[0].models[0].joints[0];
    joint.child = "missing".into();
    joint.pose.transform.translation = [100.; 3];
    assert_eq!(sdf.joint_pose("w::m::j", "").unwrap(), before);
}
