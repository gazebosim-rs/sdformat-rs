// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;

fn close(actual: Transform, expected: Transform) {
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected.translation.into_iter().chain(expected.rotation))
    {
        assert!((a - b).abs() <= 1e-12, "{a} versus {b}");
    }
}

#[test]
fn actual_native_named_joint_default_child_roundtrip() {
    let mut sdf = load_str(
        include_str!("fixtures/joint_semantic_b03.sdf"),
        "b03.sdf",
        &NoResources,
    )
    .unwrap();
    let rows: Vec<Value> = serde_json::from_str(include_str!("joint_semantic_order.json")).unwrap();
    assert_eq!(rows.len(), 4);
    for row in rows {
        assert_eq!(row["error_count"], 0);
        let name = row["joint"].as_str().unwrap();
        let child = row["child"].as_str().unwrap();
        let (scope, local) = name.rsplit_once("::").unwrap();
        let model = sdf.worlds[0]
            .models
            .iter()
            .find(|m| m.scoped_name == scope)
            .unwrap();
        let original = model.joint_by_name(local).unwrap().clone();
        assert_eq!(original.source.uri, "b03.sdf");
        assert!(original.source.line > 0 && original.source.column > 0);
        assert_eq!(original.pose.transform, Transform::default());
        let resolved = sdf.joint_pose(name, "").unwrap();
        assert_eq!(resolved, sdf.joint_pose(name, child).unwrap());
        assert_eq!(sdf.joint_pose(name, local).unwrap(), Transform::default());
        // General graph queries intentionally retain their exact local shortcut.
        assert_eq!(
            sdf.pose_of(name, &format!("{scope}::{child}")).unwrap(),
            Transform::default()
        );
        for (actual, bits) in resolved
            .translation
            .into_iter()
            .chain(resolved.rotation)
            .zip(row["pose_bits"].as_array().unwrap())
        {
            let bits = bits.as_u64().unwrap();
            let expected = f64::from_bits(bits);
            if expected == 0. {
                assert_eq!(actual, 0.); // Captured zero signs are not qualified.
            } else {
                assert_eq!(actual.to_bits(), bits);
            }
        }
        let model = sdf.worlds[0]
            .models
            .iter_mut()
            .find(|m| m.scoped_name == scope)
            .unwrap();
        let joint = model.joints.iter_mut().find(|j| j.name == local).unwrap();
        joint.child = "edited".into();
        joint.pose.transform.translation = [100.; 3];
        assert_eq!(sdf.joint_pose(name, "").unwrap(), resolved);
    }
}

#[test]
fn containing_scope_stops_before_outer_pose_and_preserves_model_alias_targets() {
    let xml = "<sdf version='1.12'><world name='w'><model name='outer'><pose>1e12 2e12 3e12 0.1 0.2 0.3</pose><link name='a'/><model name='nested'><pose>5 0 0 -1.5708 0 0</pose><link name='p'/><link name='c'><pose>3 0 0 0 0 0</pose></link><joint name='inner' type='fixed'><parent>p</parent><child>c</child></joint></model><joint name='cross' type='fixed'><pose>1 0 0 0 0 0</pose><parent>a</parent><child>nested::c</child></joint></model></world></sdf>";
    let sdf = load_str(xml, "nested.sdf", &NoResources).unwrap();
    assert_eq!(
        sdf.joint_pose("w::outer::nested::inner", "").unwrap(),
        Transform::default()
    );
    for target in ["nested", "nested::__model__"] {
        close(
            sdf.joint_pose("w::outer::cross", target).unwrap(),
            Transform {
                translation: [4., 0., 0.],
                ..Transform::default()
            },
        );
    }
    close(
        sdf.joint_pose("w::outer::cross", "nested::c").unwrap(),
        Transform {
            translation: [1., 0., 0.],
            ..Transform::default()
        },
    );
}

#[test]
fn world_joint_scope_and_zero_quaternion_sign_metadata_are_retained() {
    let xml = "<sdf version='1.12'><world name='w'><model name='m'><pose>5 0 0 0 0 0</pose><link name='p'/><link name='c'><pose rotation_format='quat_xyzw'>3 -0 0 -0 0 -0 -1</pose></link></model><joint name='j' type='fixed'><pose>1 0 0 0 0 0</pose><parent>m::p</parent><child>m::c</child></joint></world></sdf>";
    let sdf = load_str(xml, "world.sdf", &NoResources).unwrap();
    let child = &sdf.worlds[0].models[0].links[1];
    assert_eq!(child.pose.transform.rotation[3], -1.);
    assert_eq!(
        child.pose.transform.translation[1].to_bits(),
        (-0.0_f64).to_bits()
    );
    close(
        sdf.joint_pose("w::j", "").unwrap(),
        Transform {
            translation: [1., 0., 0.],
            ..Transform::default()
        },
    );
    close(
        sdf.joint_pose("w::j", "world").unwrap(),
        Transform {
            translation: [9., 0., 0.],
            rotation: [0., 0., 0., -1.],
        },
    );
    assert_eq!(sdf.joint_pose("w::j", "j").unwrap(), Transform::default());
}

#[test]
fn canonical_include_sources_and_typed_query_errors_remain_local() {
    struct Included;
    impl ResourceResolver for Included {
        fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
            Ok(Resource { canonical_uri: "canonical://joint.sdf".into(), xml: "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='fixed'><parent>a</parent><child>b</child></joint></model></sdf>".into() })
        }
    }
    let sdf = load_str(
        "<sdf version='1.12'><world name='w'><include><uri>model://m</uri></include></world></sdf>",
        "caller.sdf",
        &Included,
    )
    .unwrap();
    let joint = &sdf.worlds[0].models[0].joints[0];
    assert_eq!(joint.source.uri, "canonical://joint.sdf");
    assert_eq!(sdf.joint_pose("w::m::j", "").unwrap(), Transform::default());
    for target in ["missing", "world", "::", "w::m::a"] {
        let e = sdf.joint_pose("w::m::j", target).unwrap_err();
        assert_eq!(e.code, ErrorCode::UnknownReference);
        assert_eq!(e.source, joint.source);
    }
    assert_eq!(
        sdf.joint_pose("w::m::a", "").unwrap_err().code,
        ErrorCode::InvalidValue
    );
    assert_eq!(
        sdf.joint_pose("missing", "").unwrap_err().source.uri,
        "caller.sdf"
    );
    for pose in ["NaN 0 0 0 0 0", "0 0 0 0 0", "0 0 0 0 0 inf"] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='a'><pose>{pose}</pose></link></model></sdf>"
        );
        assert!(load_str(&xml, "bad.sdf", &NoResources).is_err());
    }
}

#[test]
fn retained_urdf_joint_query_paths_keep_defaults_and_owned_sources() {
    let import = load_urdf_str(
        include_str!("fixtures/urdf_continuous_original.urdf"),
        "input.urdf",
    )
    .unwrap();
    let model = &import.sdf.models[0];
    let joint = &model.joints[0];
    let name = format!("{}::{}", model.scoped_name, joint.name);
    let pose = import.sdf.joint_pose(&name, "").unwrap();
    assert_eq!(pose, import.sdf.joint_pose(&name, &joint.child).unwrap());
    assert_eq!(
        import.sdf.joint_pose(&name, &joint.name).unwrap(),
        Transform::default()
    );
    for target in ["", "__model__", &joint.child, &joint.parent] {
        let result = import.sdf.joint_pose(&name, target).unwrap();
        result.validate(&joint.source).unwrap();
    }
    assert!(joint.source.uri.contains("input.urdf"));
}

#[test]
fn nonfinite_relative_result_retains_existing_validation_policy() {
    let xml = "<sdf version='1.12'><model name='m'><link name='p'><pose>-1e308 0 0 0 0 0</pose></link><link name='c'><pose>1e308 0 0 0 0 0</pose></link><joint name='j' type='fixed'><parent>p</parent><child>c</child></joint></model></sdf>";
    let sdf = load_str(xml, "overflow.sdf", &NoResources).unwrap();
    let error = sdf.joint_pose("m::j", "p").unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!(error.source, sdf.models[0].joints[0].source);
    assert_eq!(
        sdf.pose_of("m::j", "m::p").unwrap_err().code,
        ErrorCode::InvalidValue
    );
}

#[test]
fn intermediate_model_references_retain_entity_implicit_and_default_edges() {
    for reference in ["nested", "nested::__model__"] {
        let xml = format!(
            "<sdf version='1.12'><world name='w'><model name='outer'><pose>10 20 30 0.2 -0.3 0.4</pose><model name='nested'><pose>2 3 4 0.1 0.2 0.3</pose><link name='p'/><link name='c'><pose>7 0 0 0 0 0</pose></link></model><frame name='f' attached_to='nested::p'><pose relative_to='{reference}'>1 2 3 0.4 -0.3 0.2</pose></frame><joint name='direct' type='fixed'><pose relative_to='{reference}'>1 2 3 0.4 -0.3 0.2</pose><parent>nested::p</parent><child>nested::c</child></joint><joint name='through_frame' type='fixed'><pose relative_to='f'>0 0 0 0 0 0</pose><parent>nested::p</parent><child>nested::c</child></joint><joint name='default' type='fixed'><parent>nested::p</parent><child>nested::c</child></joint></model></world></sdf>"
        );
        let sdf = load_str(&xml, "intermediate.sdf", &NoResources).unwrap();
        let model = &sdf.worlds[0].models[0];
        assert_eq!(model.frames[0].pose.relative_to, reference);
        assert_eq!(model.joints[0].pose.relative_to, reference);
        assert_eq!(model.joints[1].pose.relative_to, "f");
        assert!(model.models[0].links[1].pose.relative_to.is_empty());
        let expected = model.frames[0].pose.transform;
        for name in ["direct", "through_frame"] {
            close(
                sdf.joint_pose(&format!("w::outer::{name}"), reference)
                    .unwrap(),
                expected,
            );
        }
        close(
            sdf.joint_pose("w::outer::default", "nested::__model__")
                .unwrap(),
            Transform {
                translation: [7., 0., 0.],
                ..Transform::default()
            },
        );
    }
}
