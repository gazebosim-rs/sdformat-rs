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
fn projection(axis: &Axis) -> Vec<String> {
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
#[test]
fn unattached_axis_native_values_and_existing_normalization_policy() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("axis_xml.json")).unwrap();
    let fixture = load_axis_str(
        include_str!("fixtures/axis_to_element_native.xml"),
        "native-axis.xml",
    )
    .unwrap();
    let original = rows.iter().find(|row| row["id"] == "original").unwrap();
    let expected_fixture: Vec<String> = original["tokens"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().into())
        .collect();
    assert_eq!(projection(&fixture), expected_fixture);
    for row in rows {
        let expected: Vec<String> = row["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        let result = load_axis_str(row["xml"].as_str().unwrap(), "axis.xml");
        if expected[0] == "REJECT" {
            assert!(result.is_err(), "{}", row["id"]);
        } else {
            let axis = result.unwrap();
            let actual = projection(&axis);
            if row.get("native_difference").is_some() {
                assert_eq!(row["id"], "above-native-zero-threshold");
                assert_eq!(expected[1], 1e-6_f64.to_bits().to_string());
                assert_eq!(&expected[2..4], ["0", "0"]);
                assert_eq!(axis.xyz, [1., 0., 0.]);
                assert_eq!(actual[0], expected[0]);
                assert_eq!(&actual[4..], &expected[4..]);
            } else {
                assert_eq!(actual, expected, "{}", row["id"]);
            }
        }
    }
}
#[test]
fn output_preserves_limits_direction_and_native_unbounded_sentinels() {
    for value in [
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NAN,
        -f64::NAN,
        -0.,
        2.,
        1e300,
    ] {
        let mut axis = load_axis_str("<axis/>", "axis.xml").unwrap();
        axis.xyz = [0., 2., 0.];
        axis.lower = value;
        axis.upper = value;
        axis.effort = value;
        axis.velocity = value;
        let output = axis.to_xml().unwrap();
        let loaded = load_axis_str(&output, "axis.xml").unwrap();
        assert_eq!(loaded.xyz, [0., 1., 0.]);
        assert_eq!(loaded.lower.to_bits(), value.to_bits());
        assert_eq!(loaded.upper.to_bits(), value.to_bits());
        let sentinel = if value < 0. { f64::INFINITY } else { value };
        assert_eq!(loaded.effort.to_bits(), sentinel.to_bits());
        assert_eq!(loaded.velocity.to_bits(), sentinel.to_bits());
        assert_eq!(axis.xyz, [0., 2., 0.]);
    }
}
#[test]
fn standalone_references_are_unattached_but_model_admission_checks_them() {
    let xml = "<axis><xyz expressed_in='outside::frame'>0 1 0</xyz><mimic joint='unknown' axis='not_attached'><multiplier>2</multiplier></mimic></axis>";
    let axis = load_axis_str(xml, "axis.xml").unwrap();
    assert_eq!(axis.expressed_in, "outside::frame");
    assert_eq!(axis.mimic.as_ref().unwrap().axis, "not_attached");
    assert!(axis.dynamics.source.is_none());
    assert!(axis.dynamics.xml.is_none());
    let output = axis.to_xml().unwrap();
    let loaded = load_axis_str(&output, "axis.xml").unwrap();
    assert_eq!(loaded.mimic, axis.mimic);
    let world = format!(
        "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child>{output}</joint></model></sdf>"
    );
    assert!(load_str(&world, "model.sdf", &NoResources).is_err());
}
#[test]
fn output_escapes_owned_references_and_rejects_lossy_attributes() {
    let mut axis = load_axis_str("<axis/>", "axis.xml").unwrap();
    axis.expressed_in = "frame & \"quoted\"".into();
    axis.mimic = Some(AxisMimic {
        joint: "leader & \"quoted\"".into(),
        axis: "axis2".into(),
        multiplier: 5.,
        offset: 1.,
        reference: 2.,
    });
    let xml = axis.to_xml().unwrap();
    assert!(xml.contains("&amp;") && xml.contains("&quot;"));
    let loaded = load_axis_str(&xml, "axis.xml").unwrap();
    assert_eq!(loaded.expressed_in, axis.expressed_in);
    assert_eq!(loaded.mimic, axis.mimic);
    for value in [
        " padded", "padded ", "a\tb", "a\nb", "a\rb", "a\0b", "\u{fffe}", "\u{ffff}",
    ] {
        let mut bad = axis.clone();
        bad.expressed_in = value.into();
        assert_eq!(bad.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        let mut bad = axis.clone();
        bad.mimic.as_mut().unwrap().joint = value.into();
        assert_eq!(bad.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        let mut bad = axis.clone();
        bad.mimic.as_mut().unwrap().axis = value.into();
        assert_eq!(bad.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    axis.mimic.as_mut().unwrap().joint.clear();
    assert_eq!(axis.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
}
#[test]
fn output_rejects_unreadable_numeric_states() {
    let original = load_axis_str("<axis/>", "axis.xml").unwrap();
    for direction in [
        [0.; 3],
        [f64::from(1e-6_f32), 0., 0.],
        [f64::INFINITY, 0., 0.],
        [f64::NAN, 0., 0.],
        [1e-210, 1e100, 0.],
    ] {
        let mut axis = original.clone();
        axis.xyz = direction;
        assert_eq!(axis.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(1),
    ] {
        for field in 0..6 {
            let mut axis = original.clone();
            match field {
                0 => axis.dynamics.damping = value,
                1 => axis.dynamics.friction = value,
                2 => axis.dynamics.spring_reference = value,
                3 => axis.dynamics.spring_stiffness = value,
                4 => axis.stiffness = value,
                5 => axis.dissipation = value,
                _ => unreachable!(),
            }
            assert_eq!(axis.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
        for field in 0..3 {
            let mut axis = original.clone();
            axis.mimic = Some(AxisMimic {
                joint: "leader".into(),
                axis: "axis".into(),
                multiplier: 1.,
                offset: 0.,
                reference: 0.,
            });
            let mimic = axis.mimic.as_mut().unwrap();
            match field {
                0 => mimic.multiplier = value,
                1 => mimic.offset = value,
                2 => mimic.reference = value,
                _ => unreachable!(),
            }
            assert_eq!(axis.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    for field in 0..4 {
        let mut axis = original.clone();
        let tiny = f64::from_bits(1);
        match field {
            0 => axis.lower = tiny,
            1 => axis.upper = tiny,
            2 => axis.effort = tiny,
            3 => axis.velocity = tiny,
            _ => unreachable!(),
        }
        assert_eq!(axis.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    let mut above = original.clone();
    above.xyz = [1e-6, 0., 0.];
    assert!(above.to_xml().is_ok());
    let mut axis = original;
    axis.xyz = [1e-310, 1e-5, 0.];
    let xml = axis.to_xml().unwrap();
    assert!(load_axis_str(&xml, "axis.xml").is_ok());
}
#[test]
fn standalone_admission_rejects_wrong_roots_and_keeps_document_version_gates() {
    for xml in [
        "<axis",
        "<axis2/>",
        "<joint/>",
        "<axis extra='x'/>",
        "<axis><use_parent_model_frame>true</use_parent_model_frame></axis>",
    ] {
        assert!(load_axis_str(xml, "bad.xml").is_err(), "{xml}");
    }
    let axis = "<axis><mimic joint='leader'><multiplier>2</multiplier></mimic></axis>";
    assert!(load_axis_str(axis, "axis.xml").is_ok());
    for (axis, expected) in [
        (
            "<axis><xyz expressed_in='unknown'>0 1 0</xyz></axis>",
            ErrorCode::JointAxisExpressedInInvalid,
        ),
        (
            "<axis><mimic joint='unknown'/></axis>",
            ErrorCode::JointAxisMimicInvalid,
        ),
        (
            "<axis><mimic joint='j' axis='bad'/></axis>",
            ErrorCode::JointAxisMimicInvalid,
        ),
    ] {
        assert!(load_axis_str(axis, "axis.xml").is_ok());
        let model = format!(
            "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child>{axis}</joint></model></sdf>"
        );
        assert_eq!(
            load_str(&model, "model.sdf", &NoResources)
                .unwrap_err()
                .code,
            expected
        );
    }
    let old = format!(
        "<sdf version='1.10'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child>{axis}</joint></model></sdf>"
    );
    assert_eq!(
        load_str(&old, "old.sdf", &NoResources).unwrap_err().code,
        ErrorCode::Unsupported
    );
}
