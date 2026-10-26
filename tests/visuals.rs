// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

fn document(content: &str) -> Result<Sdf, Diagnostic> {
    load_str(
        &format!("<sdf version='1.12'><model name='m'>{content}</model></sdf>"),
        "visuals",
        &NoResources,
    )
}
#[test]
fn visual_geometry_is_optional_and_plugins_keep_original_content() {
    let xml = "<link name='l'><visual name='absent'/><visual name='empty'><geometry/></visual><visual name='shape'><pose relative_to='f'>1 2 3 0 0 0</pose><geometry><sphere><radius>2</radius></sphere></geometry><plugin name='p' filename='f'><x:key>value</x:key></plugin></visual></link><frame name='f'><pose>10 0 0 0 0 0</pose></frame>";
    let sdf = document(xml).unwrap();
    let model = &sdf.models[0];
    let link = &model.links[0];
    assert_eq!(link.visuals.len(), 3);
    assert!(link.visuals[0].geometry.is_none());
    assert!(link.visuals[1].geometry.is_none());
    let visual = &link.visuals[2];
    assert_eq!(visual.geometry, Some(Geometry::Sphere { radius: 2. }));
    assert_eq!(visual.pose.relative_to, "f");
    assert_eq!(visual.source.uri, "visuals");
    assert_eq!(
        visual.plugins[0].xml,
        "<plugin name='p' filename='f'><x:key>value</x:key></plugin>"
    );
    assert_eq!(
        sdf.resolve_pose(&visual.pose, &model.scoped_name, &link.scoped_name, "m")
            .unwrap()
            .translation,
        [11., 2., 3.]
    );
    // Collision and visual names occupy separate collections, matching native.
    document("<link name='l'><collision name='same'><geometry><box/></geometry></collision><visual name='same'/></link>").unwrap();
}
#[test]
fn visual_validation_and_explicit_remaining_schema_contract() {
    // The former material boundary input now loads native default metadata.
    let material = document("<link name='l'><visual name='v'><material/></visual></link>").unwrap();
    assert_eq!(
        material.models[0].links[0].visuals[0]
            .material
            .as_deref()
            .unwrap()
            .shader,
        ShaderType::Pixel
    );

    for (visual, code) in [
        (
            "<visual name='v'/><visual name='v'/>",
            ErrorCode::DuplicateName,
        ),
        ("<visual/>", ErrorCode::Missing),
        (
            "<visual name='v'><geometry foo='x'/></visual>",
            ErrorCode::Unsupported,
        ),
        (
            "<visual name='v'><geometry><box/><sphere/></geometry></visual>",
            ErrorCode::InvalidValue,
        ),
    ] {
        let xml = format!("<link name='l'>{visual}</link>");
        assert_eq!(document(&xml).unwrap_err().code, code, "{visual}");
    }
    let transparent = document(
        "<link name='l'><visual name='v'><transparency>0.5</transparency></visual></link>",
    )
    .unwrap();
    assert_eq!(
        transparent.models[0].links[0].visuals[0]
            .properties
            .transparency,
        0.5
    );
    let sdf = document("<link name='l'><visual name='v'><geometry><sphere><radius>-1</radius></sphere></geometry></visual></link>").unwrap();
    assert!(matches!(
        sdf.models[0].links[0].visuals[0].geometry,
        Some(Geometry::Sphere { radius: 1. })
    ));
}
#[test]
fn both_joint_axes_keep_omission_defaults_limits_and_scope_validation() {
    for (kind, expected) in [
        ("universal", JointKind::Universal),
        ("revolute2", JointKind::Revolute2),
    ] {
        let prefix = format!(
            "<link name='a'/><link name='b'/><joint name='j' type='{kind}'><parent>a</parent><child>b</child>"
        );
        let absent = document(&format!("{prefix}</joint>")).unwrap();
        let joint = &absent.models[0].joints[0];
        assert_eq!(joint.kind, expected);
        assert!(joint.axis.is_none());
        assert!(joint.axis2.is_none());
        let present = document(&format!("{prefix}<axis2/></joint>")).unwrap();
        assert_eq!(
            present.models[0].joints[0].axis2.as_ref().unwrap().xyz,
            [0., 0., 1.]
        );
    }
    let prefix = "<link name='a'/><link name='b'/><joint name='j' type='fixed'><parent>a</parent><child>b</child>";
    let sdf=document(&format!("{prefix}<axis2><xyz>2 0 0</xyz><limit><lower>-1</lower><upper>2</upper><effort>-3</effort><velocity>4</velocity></limit></axis2></joint>")).unwrap();
    let axis = sdf.models[0].joints[0].axis2.as_ref().unwrap();
    assert_eq!(axis.xyz, [1., 0., 0.]);
    assert_eq!(
        (axis.lower, axis.upper, axis.effort, axis.velocity),
        (-1., 2., f64::INFINITY, 4.)
    );
    for (axis, code) in [
        ("<xyz>0 0 0</xyz>", ErrorCode::InvalidValue),
        (
            "<xyz expressed_in='missing'>0 1 0</xyz>",
            ErrorCode::JointAxisExpressedInInvalid,
        ),
        (
            "<xyz expressed_in='world'>0 1 0</xyz>",
            ErrorCode::JointAxisExpressedInInvalid,
        ),
    ] {
        assert_eq!(
            document(&format!("{prefix}<axis2>{axis}</axis2></joint>"))
                .unwrap_err()
                .code,
            code,
            "{axis}"
        );
    }
}

#[test]
fn explicit_non_frame_validation_is_recursive_and_separate_from_loading() {
    document("<link name='l'/>")
        .unwrap()
        .validate_non_frame_poses()
        .unwrap();
    let xml = "<sdf version='1.12'><world name='w'><model name='outer'><model name='inner'><link name='l'><inertial><pose relative_to='missing'/></inertial></link></model></model></world></sdf>";
    let sdf = load_str(xml, "inertial-policy", &NoResources).unwrap();
    assert_eq!(
        sdf.worlds[0].models[0].models[0].links[0]
            .inertial
            .pose
            .relative_to,
        "missing"
    );
    assert_eq!(
        sdf.validate_non_frame_poses().unwrap_err().code,
        ErrorCode::UnknownReference
    );
    // This is an explicit consumer policy: native inertial loading ignores
    // relative_to rather than exposing a separate inertial SemanticPose.
    let valid = "<sdf version='1.12'><world name='w'><model name='outer'><model name='inner'><link name='l'><visual name='v'/><collision name='c'><geometry><box/></geometry></collision><sensor name='s' type='imu'/></link></model></model></world></sdf>";
    load_str(valid, "valid-nested", &NoResources)
        .unwrap()
        .validate_non_frame_poses()
        .unwrap();
    load_str(
        "<sdf version='1.12'><world name='empty'/></sdf>",
        "empty-world",
        &NoResources,
    )
    .unwrap()
    .validate_non_frame_poses()
    .unwrap();
}
