// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
fn hex(value: &str) -> String {
    if value.is_empty() {
        return "-".into();
    }
    value
        .as_bytes()
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect()
}
fn projection(value: &ContactSensor) -> Vec<String> {
    let mut output = vec!["OK".into(), value.collisions.len().to_string()];
    output.extend(value.collisions.iter().map(|v| hex(v)));
    output.push(hex(&value.topic));
    output
}
fn sensor(xml: &str) -> Sensor {
    let mut value = Sensor::default();
    value
        .load_element_str(xml, "sensor.xml", &NoResources)
        .unwrap();
    value
}
fn sensor_projection(value: &Sensor) -> Vec<String> {
    let mut output = vec![
        "OK".into(),
        hex(&value.name),
        hex(&value.kind),
        hex(&value.frame_id),
        hex(&value.topic),
        hex(&value.pose.relative_to),
        value.update_rate.to_bits().to_string(),
        u8::from(value.enable_metrics).to_string(),
        u8::from(value.always_on).to_string(),
        u8::from(value.visualize).to_string(),
    ];
    output.extend(
        value
            .pose
            .transform
            .translation
            .into_iter()
            .chain(value.pose.transform.rotation)
            .map(|v| v.to_bits().to_string()),
    );
    output.extend(projection(value.contact().unwrap()).into_iter().skip(1));
    output
}
fn compare(actual: &[String], expected: &[String], envelope: bool) {
    let count_index = if envelope { 17 } else { 1 };
    let count: usize = expected[count_index].parse().unwrap();
    assert_eq!(expected.len(), if envelope { 19 } else { 3 } + count);
    assert_eq!(actual.len(), expected.len());
    for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
        if envelope && (10..17).contains(&i) {
            let a = f64::from_bits(a.parse().unwrap());
            let b = f64::from_bits(b.parse().unwrap());
            assert!((a - b).abs() <= 1e-3, "pose {i}: {a} != {b}");
        } else {
            assert_eq!(a, b, "token {i}");
        }
    }
}
fn rows() -> Vec<Value> {
    serde_json::from_str(include_str!("contact_xml.json")).unwrap()
}
fn tokens(row: &Value) -> Vec<String> {
    row["tokens"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().into())
        .collect()
}
fn export(name: &str, xml: &str) {
    if let Some(directory) = std::env::var_os("SDF_CONTACT_XML_EXPORTS") {
        std::fs::write(
            std::path::Path::new(&directory).join(format!("{name}.xml")),
            xml,
        )
        .unwrap();
    }
}
#[test]
fn native_contact_values_and_envelopes_roundtrip() {
    for row in rows() {
        let expected = tokens(&row);
        assert_eq!(expected[0], "OK");
        let input = row["xml"].as_str().unwrap();
        let output = if row["mode"] == "config" {
            let value = load_contact_sensor_str(input, "native.xml").unwrap();
            compare(&projection(&value), &expected, false);
            let output = value.to_xml().unwrap();
            compare(
                &projection(&load_contact_sensor_str(&output, "rust.xml").unwrap()),
                &expected,
                false,
            );
            output
        } else {
            let value = sensor(input);
            compare(&sensor_projection(&value), &expected, true);
            let output = value.to_xml().unwrap();
            compare(&sensor_projection(&sensor(&output)), &expected, true);
            output
        };
        export(row["id"].as_str().unwrap(), &output);
    }
}
#[test]
fn current_values_and_declaration_order_override_old_input_without_mutation() {
    let mut value = load_contact_sensor_str(
        "<contact><collision>a</collision><collision>b</collision><topic>/topic</topic></contact>",
        "old.xml",
    )
    .unwrap();
    let source = value.source.clone();
    value.collisions[0] = "changed  collision".into();
    value.collisions.push("a".into());
    value.topic = "topic  changed".into();
    let output = value.to_xml().unwrap();
    let rows = rows();
    let row = rows.iter().find(|r| r["id"] == "typed_edits").unwrap();
    compare(
        &projection(&load_contact_sensor_str(&output, "edited.xml").unwrap()),
        &tokens(row),
        false,
    );
    assert_eq!(value.source, source);
    export("edited-authoritative", &output);
}
#[test]
fn unrepresentable_text_and_reader_boundaries_are_explicit() {
    let mut value = ContactSensor::default();
    value.collisions.clear();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    for field in 0..2 {
        for bad in [
            "", " padded", "padded ", "a\tb", "a\nb", "a\rb", "a\0b", "\u{fffe}", "\u{ffff}",
        ] {
            let mut value = ContactSensor::default();
            if field == 0 {
                value.collisions[0] = bad.into();
            } else {
                value.topic = bad.into();
            }
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
        let mut value = ContactSensor::default();
        if field == 0 {
            value.collisions[0] = "a  ]]>b".into();
        } else {
            value.topic = "a  ]]>b".into();
        }
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
    for xml in [
        "<",
        "<surface/>",
        "<contact unknown='x'/>",
        "<contact><ode/></contact>",
        "<contact><collision unknown='x'>c</collision></contact>",
        "<contact><topic/></contact>",
    ] {
        assert!(load_contact_sensor_str(xml, "bad.xml").is_err());
    }
    let error =
        load_contact_sensor_str("\n<contact><unknown/></contact>", "caller.xml").unwrap_err();
    assert_eq!(error.source.uri, "caller.xml");
    assert_eq!(error.source.line, 2);
}
#[test]
fn sensor_payload_and_joint_boundaries_stay_explicit() {
    let mut value = sensor("<sensor name='s' type='contact'/>");
    value.kind = "imu".into();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    value.kind = "contact".into();
    let SensorConfig::Contact(config) = &mut value.config else {
        panic!("contact")
    };
    config.topic.clear();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    for extra in [
        "<plugin name='p' filename='f'/>",
        "<alwaysOn>true</alwaysOn>",
        "<gz_frame_id>legacy</gz_frame_id>",
    ] {
        let value = sensor(&format!("<sensor name='s' type='contact'>{extra}</sensor>"));
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
    let xml = "<joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='s' type='contact'/></joint>";
    assert_eq!(
        load_joint_str(xml, "joint.xml").unwrap_err().code,
        ErrorCode::Unsupported
    );
    let mut joint = load_joint_str(
        "<joint name='j' type='fixed'><parent>a</parent><child>b</child></joint>",
        "joint.xml",
    )
    .unwrap();
    joint
        .sensors
        .push(sensor("<sensor name='s' type='contact'/>"));
    assert_eq!(joint.to_xml().unwrap_err().code, ErrorCode::Unsupported);
}
