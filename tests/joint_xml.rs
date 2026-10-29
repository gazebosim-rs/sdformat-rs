// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;

fn hex(text: &str) -> String {
    if text.is_empty() {
        return "-".into();
    }
    text.as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn axis_projection(axis: &Axis) -> Vec<String> {
    let mut values = vec!["OK".into()];
    for value in axis.xyz {
        values.push(value.to_bits().to_string());
    }
    values.push(hex(&axis.expressed_in));
    for value in [
        axis.dynamics.damping,
        axis.dynamics.friction,
        axis.dynamics.spring_reference,
        axis.dynamics.spring_stiffness,
        axis.lower,
        axis.upper,
        axis.effort,
        axis.velocity,
        axis.stiffness,
        axis.dissipation,
    ] {
        values.push(value.to_bits().to_string());
    }
    values.push(u8::from(axis.mimic.is_some()).to_string());
    if let Some(mimic) = &axis.mimic {
        values.push(hex(&mimic.joint));
        values.push(hex(&mimic.axis));
        for value in [mimic.multiplier, mimic.offset, mimic.reference] {
            values.push(value.to_bits().to_string());
        }
    }
    values
}

fn projection(joint: &Joint) -> Vec<String> {
    let kind = match joint.kind {
        JointKind::Ball => 1,
        JointKind::Continuous => 2,
        JointKind::Fixed => 3,
        JointKind::Gearbox => 4,
        JointKind::Prismatic => 5,
        JointKind::Revolute => 6,
        JointKind::Revolute2 => 7,
        JointKind::Screw => 8,
        JointKind::Universal => 9,
    };
    let mut values = vec![
        "OK".into(),
        hex(&joint.name),
        kind.to_string(),
        hex(&joint.parent),
        hex(&joint.child),
        hex(&joint.pose.relative_to),
    ];
    for value in joint
        .pose
        .transform
        .translation
        .into_iter()
        .chain(joint.pose.transform.rotation)
    {
        values.push(value.to_bits().to_string());
    }
    values.push(joint.screw_thread_pitch.to_bits().to_string());
    for axis in [&joint.axis, &joint.axis2] {
        values.push(u8::from(axis.is_some()).to_string());
        if let Some(axis) = axis {
            values.extend(axis_projection(axis).into_iter().skip(1));
        }
    }
    values
}
fn compare(actual: &[String], expected: &[String]) {
    assert_eq!(actual.len(), expected.len());
    for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
        if (6..13).contains(&i) {
            let a = f64::from_bits(a.parse().unwrap());
            let e = f64::from_bits(e.parse().unwrap());
            assert!((a - e).abs() <= 1e-3, "pose component {i}: {a} != {e}");
        } else {
            assert_eq!(a, e, "token {i}");
        }
    }
}
fn joint() -> Joint {
    load_joint_str(
        "<joint name='j' type='revolute'><parent>parent</parent><child>child</child></joint>",
        "joint.xml",
    )
    .unwrap()
}
#[test]
fn native_metadata_and_two_way_roundtrip() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("joint_xml.json")).unwrap();
    for row in rows {
        let expected: Vec<String> = row["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        let result = load_joint_str(row["xml"].as_str().unwrap(), "native.xml");
        if expected[0] == "REJECT" {
            assert!(result.is_err(), "{}", row["id"]);
            continue;
        }
        let loaded = result.unwrap();
        compare(&projection(&loaded), &expected);
        let xml = loaded.to_xml().unwrap();
        let again = load_joint_str(&xml, "rust.xml").unwrap();
        compare(&projection(&again), &expected);
        if row["id"] == "native-subset" {
            let fixture = load_joint_str(
                include_str!("fixtures/joint_to_element_native_subset.xml"),
                "fixture.xml",
            )
            .unwrap();
            compare(&projection(&fixture), &expected);
            if let Some(path) = std::env::var_os("SDF_JOINT_XML_EXPORT") {
                std::fs::write(path, &xml).unwrap();
            }
        }
    }
}
#[test]
fn owned_values_escape_and_mutations_are_serialized() {
    let mut value = joint();
    value.name = "j & \"quoted\"".into();
    value.parent = "parent & \"quoted\"".into();
    value.child = "child & \"quoted\"".into();
    value.pose.relative_to = "frame & \"quoted\"".into();
    let xml = value.to_xml().unwrap();
    assert!(xml.contains("&amp;") && xml.contains("&quot;"));
    compare(
        &projection(&load_joint_str(&xml, "rust.xml").unwrap()),
        &projection(&value),
    );
    value.parent = "new_parent".into();
    assert_eq!(
        load_joint_str(&value.to_xml().unwrap(), "rust.xml")
            .unwrap()
            .parent,
        "new_parent"
    );
    assert_eq!(value.parent, "new_parent");
}
#[test]
fn unsupported_payload_is_never_silently_lost() {
    for payload in [
        "<sensor name='s' type='imu'/>",
        "<physics/>",
        "<axis2/>",
        "<screw_thread_pitch>2</screw_thread_pitch>",
    ] {
        let xml = format!(
            "<joint name='j' type='revolute'><parent>a</parent><child>b</child>{payload}</joint>"
        );
        assert_eq!(
            load_joint_str(&xml, "joint.xml").unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
    let mut value = joint();
    value.physics_xml = Some("<physics/>".into());
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    value.physics_xml = None;
    value.axis2 = Some(load_axis_str("<axis/>", "axis.xml").unwrap());
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    value.axis2 = None;
    value.screw_thread_pitch = 2.;
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
}
#[test]
fn invalid_local_states_and_xml_are_rejected() {
    for xml in [
        "<",
        "<axis/>",
        "<joint/>",
        "<joint name='j' type='fixed'><parent>a</parent></joint>",
    ] {
        assert!(load_joint_str(xml, "bad.xml").is_err());
    }
    for field in 0..4 {
        for bad in [
            " padded", "padded ", "a\tb", "a\nb", "a\rb", "a\0b", "\u{fffe}", "\u{ffff}",
        ] {
            let mut value = joint();
            match field {
                0 => value.name = bad.into(),
                1 => value.parent = bad.into(),
                2 => value.child = bad.into(),
                _ => value.pose.relative_to = bad.into(),
            };
            assert!(value.to_xml().is_err());
        }
    }
    for bad in ["", "world", "__reserved__", "a::b"] {
        let mut value = joint();
        value.name = bad.into();
        assert!(value.to_xml().is_err());
    }
    for (parent, child) in [
        ("a", "a"),
        ("world", "world"),
        ("__root__", "child"),
        ("parent", "__root__"),
        ("", "child"),
        ("parent", ""),
    ] {
        let mut value = joint();
        value.parent = parent.into();
        value.child = child.into();
        assert!(value.to_xml().is_err());
    }
    for bad in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        let mut value = joint();
        value.kind = JointKind::Screw;
        value.screw_thread_pitch = bad;
        assert!(value.to_xml().is_err());
    }
    for bad in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        let mut value = joint();
        value.pose.transform.translation[0] = bad;
        assert!(value.to_xml().is_err());
    }
    let mut value = joint();
    value.pose.transform.rotation = [0.; 4];
    assert!(value.to_xml().is_err());
    value = joint();
    value.pose.transform.rotation[0] = f64::from_bits(1);
    assert!(value.to_xml().is_err());
    value = joint();
    value.axis = Some(load_axis_str("<axis/>", "axis.xml").unwrap());
    value.axis.as_mut().unwrap().xyz = [0.; 3];
    assert!(value.to_xml().is_err());
    value = joint();
    value.axis =
        Some(load_axis_str("<axis><mimic joint='j' axis='axis'/></axis>", "axis.xml").unwrap());
    assert_eq!(
        value.to_xml().unwrap_err().code,
        ErrorCode::JointAxisMimicInvalid
    );
    value.axis.as_mut().unwrap().mimic.as_mut().unwrap().axis = "invalid".into();
    assert_eq!(
        value.to_xml().unwrap_err().code,
        ErrorCode::JointAxisMimicInvalid
    );
}
#[test]
fn unattached_references_defer_graph_validation() {
    let xml = "<joint name='j' type='revolute'><parent>missing_parent</parent><child>missing_child</child><pose relative_to='outside::frame'>0 0 0 0 0 0</pose><axis><xyz expressed_in='outside::frame'>0 1 0</xyz><mimic joint='unknown' axis='axis2'/></axis></joint>";
    let value = load_joint_str(xml, "joint.xml").unwrap();
    assert_eq!(
        value.axis.as_ref().unwrap().mimic.as_ref().unwrap().joint,
        "unknown"
    );
    assert!(load_joint_str(&value.to_xml().unwrap(), "roundtrip.xml").is_ok());
    assert!(
        load_str(
            &format!("<sdf version='1.12'><model name='m'><link name='a'/>{xml}</model></sdf>"),
            "model.sdf",
            &NoResources
        )
        .is_err()
    );
}

// Supported assertions from DOMJoint.ToElement. XML setup replaces default DOM
// construction; sensor/Element contracts deliberately remain outside this port.
#[test]
fn original_to_element_metadata_subset() {
    let mut value = load_joint_str(
        include_str!("fixtures/joint_to_element_native_subset.xml"),
        "native.xml",
    )
    .unwrap();
    let xml = value.to_xml().unwrap();
    assert!(!xml.is_empty());
    let again = load_joint_str(&xml, "rust.xml").unwrap();
    for (a, b) in again
        .pose
        .transform
        .translation
        .into_iter()
        .zip([-1., -2., -3.])
    {
        assert!((a - b).abs() <= 1e-3);
    }
    let expected =
        glam::DQuat::from_euler(glam::EulerRot::XYZ, 0., std::f64::consts::PI, 0.).to_array();
    for (a, b) in again.pose.transform.rotation.into_iter().zip(expected) {
        assert!((a - b).abs() <= 1e-3);
    }
    assert_eq!(again.pose.relative_to, "link");
    assert_eq!(again.name, "test_joint");
    assert_eq!(again.parent, "parent");
    assert_eq!(again.child, "child");
    assert_eq!(again.kind, JointKind::Ball);
    assert!(again.axis.is_some());
    assert!(again.axis2.is_some());
    assert_eq!(again.axis.as_ref().unwrap().xyz, [1., 0., 0.]);
    assert_eq!(again.axis2.as_ref().unwrap().xyz, [0., 1., 0.]);
    value.parent = "new_parent".into();
    let updated = value.to_xml().unwrap();
    assert!(!updated.is_empty());
    assert_eq!(
        load_joint_str(&updated, "updated.xml").unwrap().parent,
        "new_parent"
    );
}
#[test]
fn local_admission_and_retained_sensor_rejection() {
    for (parent, child, axis) in [
        ("a", "a", ""),
        ("__root__", "b", ""),
        ("a", "world", ""),
        ("a", "b", "<axis><mimic joint='j' axis='axis'/></axis>"),
        (
            "a",
            "b",
            "<axis><mimic joint='other' axis='invalid'/></axis>",
        ),
    ] {
        let xml = format!(
            "<joint name='j' type='revolute'><parent>{parent}</parent><child>{child}</child>{axis}</joint>"
        );
        assert!(load_joint_str(&xml, "local.xml").is_err());
    }
    let sdf=load_str("<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='s' type='imu'/></joint></model></sdf>","model.xml",&NoResources).unwrap();
    assert_eq!(sdf.models[0].joints[0].sensors.len(), 1);
    assert_eq!(
        sdf.models[0].joints[0].to_xml().unwrap_err().code,
        ErrorCode::Unsupported
    );
    let padded=load_joint_str("<joint name='j' type='fixed'><parent>  parent  </parent><child> child </child><pose relative_to=' frame '>0 0 0 0 0 0</pose><axis><xyz expressed_in=' frame '>1 0 0</xyz></axis></joint>","trim.xml").unwrap();
    assert_eq!(padded.parent, "parent");
    assert_eq!(padded.child, "child");
    assert_eq!(padded.pose.relative_to, "frame");
    assert_eq!(padded.axis.unwrap().expressed_in, "frame");
}

#[test]
fn standalone_boundary_diagnostic_preserves_input_location() {
    let error = load_joint_str(
        "\n<joint name='j' type='fixed'><parent>a</parent><child>a</child></joint>",
        "caller/joint.xml",
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!(error.source.uri, "caller/joint.xml");
    assert_eq!(error.source.line, 2);
    assert_eq!(error.source.column, 1);
}
