// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
fn projection(value: &ForceTorque) -> Vec<String> {
    let frame = match value.frame {
        ForceTorqueFrame::Parent => 1,
        ForceTorqueFrame::Child => 2,
        ForceTorqueFrame::Sensor => 3,
    };
    let direction = match value.measure_direction {
        ForceTorqueMeasureDirection::ParentToChild => 1,
        ForceTorqueMeasureDirection::ChildToParent => 2,
    };
    let mut result = vec!["OK".into(), frame.to_string(), direction.to_string()];
    for noise in value.force.iter().chain(&value.torque) {
        result.push(
            match noise.kind {
                NoiseKind::None => "0",
                NoiseKind::Gaussian => "1",
                NoiseKind::GaussianQuantized => "2",
            }
            .into(),
        );
        for n in [
            noise.mean,
            noise.stddev,
            noise.bias_mean,
            noise.bias_stddev,
            noise.dynamic_bias_stddev,
            noise.dynamic_bias_correlation_time,
            noise.precision,
        ] {
            result.push(n.to_bits().to_string());
        }
    }
    result
}
#[test]
fn native_configuration_values_roundtrip() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("force_torque_xml.json")).unwrap();
    for row in rows.iter().filter(|row| row["mode"] == "config") {
        let tokens: Vec<String> = row["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        let loaded = load_force_torque_str(row["xml"].as_str().unwrap(), "native.xml");
        if tokens[0] == "REJECT" {
            assert_eq!(tokens.len(), 1);
            assert!(loaded.is_err());
            continue;
        }
        assert_eq!(tokens.len(), 51);
        let loaded = loaded.unwrap();
        assert_eq!(projection(&loaded), tokens, "{}", row["id"]);
        let output = loaded.to_xml().unwrap();
        let again = load_force_torque_str(&output, "rust.xml").unwrap();
        assert_eq!(projection(&again), tokens, "{}", row["id"]);
        if row["id"] == "original" {
            let original = load_force_torque_str(
                include_str!("fixtures/force_torque_to_element_native.xml"),
                "fixture.xml",
            )
            .unwrap();
            assert_eq!(projection(&original), tokens);
            if let Some(path) = std::env::var_os("SDF_FORCE_TORQUE_XML_EXPORT") {
                std::fs::write(path, &output).unwrap();
            }
        }
    }
}
#[test]
fn numeric_reader_boundary_and_provenance_are_explicit() {
    for xml in [
        "<",
        "<axis/>",
        "<force_torque><force><x><noise type='invalid'/></x></force></force_torque>",
    ] {
        assert!(load_force_torque_str(xml, "bad.xml").is_err());
    }
    let original = ForceTorque::default();
    for index in 0..7 {
        for n in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
            let mut value = original.clone();
            let noise = &mut value.force[0];
            match index {
                0 => noise.mean = n,
                1 => noise.stddev = n,
                2 => noise.bias_mean = n,
                3 => noise.bias_stddev = n,
                4 => noise.dynamic_bias_stddev = n,
                5 => noise.dynamic_bias_correlation_time = n,
                _ => noise.precision = n,
            };
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    for index in 0..3 {
        let mut value = original.clone();
        let noise = &mut value.torque[0];
        match index {
            0 => noise.stddev = -1.,
            1 => noise.bias_stddev = -1.,
            _ => noise.dynamic_bias_stddev = -1.,
        };
        assert!(value.to_xml().is_err());
    }
    let mut value = load_force_torque_str("\n<force_torque/>", "caller.xml").unwrap();
    assert_eq!(value.source.as_ref().unwrap().uri, "caller.xml");
    assert_eq!(value.source.as_ref().unwrap().line, 2);
    let source = value.source.clone();
    value.measure_direction = ForceTorqueMeasureDirection::ParentToChild;
    let loaded = load_force_torque_str(&value.to_xml().unwrap(), "edited.xml").unwrap();
    assert_eq!(
        loaded.measure_direction,
        ForceTorqueMeasureDirection::ParentToChild
    );
    assert_eq!(value.source, source);
}

fn hex(value: &str) -> String {
    if value.is_empty() {
        return "-".into();
    }
    value
        .as_bytes()
        .iter()
        .map(|n| format!("{n:02x}"))
        .collect()
}
fn sensor_projection(value: &Sensor) -> Vec<String> {
    let mut result = vec![
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
    result.extend(
        value
            .pose
            .transform
            .translation
            .into_iter()
            .chain(value.pose.transform.rotation)
            .map(|n| n.to_bits().to_string()),
    );
    result.extend(
        projection(value.force_torque().unwrap())
            .into_iter()
            .skip(1),
    );
    result
}
fn compare_sensor(actual: &[String], expected: &[String]) {
    assert_eq!(actual.len(), 66);
    assert_eq!(expected.len(), 66);
    for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
        if (9..16).contains(&i) {
            let a = f64::from_bits(a.parse().unwrap());
            let b = f64::from_bits(b.parse().unwrap());
            assert!((a - b).abs() <= 1e-3, "pose {i}: {a} != {b}");
        } else {
            assert_eq!(a, b, "token {i}");
        }
    }
}
fn sensor(xml: &str) -> Sensor {
    let mut sensor = Sensor::default();
    sensor
        .load_element_str(xml, "sensor.xml", &NoResources)
        .unwrap();
    sensor
}
#[test]
fn native_sensor_and_joint_metadata_roundtrips() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("force_torque_xml.json")).unwrap();
    for row in rows.iter().filter(|row| row["mode"] != "config") {
        let expected: Vec<String> = row["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        assert_eq!(expected[0], "OK");
        if row["mode"] == "sensor" {
            let value = sensor(row["xml"].as_str().unwrap());
            compare_sensor(&sensor_projection(&value), &expected[1..]);
            let xml = value.to_xml().unwrap();
            compare_sensor(&sensor_projection(&sensor(&xml)), &expected[1..]);
            if let Some(directory) = std::env::var_os("SDF_FORCE_TORQUE_XML_EXPORTS") {
                std::fs::write(
                    std::path::Path::new(&directory)
                        .join(format!("{}.xml", row["id"].as_str().unwrap())),
                    xml,
                )
                .unwrap();
            }
        } else {
            let joint = load_joint_str(row["xml"].as_str().unwrap(), "joint.xml").unwrap();
            assert_eq!(expected.len(), 2 + 66 * joint.sensors.len());
            assert_eq!(expected[1], joint.sensors.len().to_string());
            let output = joint.to_xml().unwrap();
            let loaded = load_joint_str(&output, "loaded.xml").unwrap();
            assert_eq!(loaded.sensors.len(), joint.sensors.len());
            for (i, (source, loaded)) in joint.sensors.iter().zip(&loaded.sensors).enumerate() {
                let expected = &expected[2 + 66 * i..2 + 66 * (i + 1)];
                compare_sensor(&sensor_projection(source), expected);
                compare_sensor(&sensor_projection(loaded), expected);
            }
            if let Some(directory) = std::env::var_os("SDF_FORCE_TORQUE_XML_EXPORTS") {
                std::fs::write(
                    std::path::Path::new(&directory)
                        .join(format!("{}.xml", row["id"].as_str().unwrap())),
                    output,
                )
                .unwrap();
            }
        }
    }
}
#[test]
fn sensor_envelope_edits_escape_and_unsupported_payloads_reject() {
    let mut value = sensor("<sensor name='s' type='force_torque'/>");
    value.name = "s & \"quoted\"".into();
    value.frame_id = "frame & \"quoted\"".into();
    value.topic = "/topic&value".into();
    value.pose.relative_to = "frame & \"quoted\"".into();
    value.update_rate = -1.25;
    value.always_on = true;
    value.visualize = true;
    value.enable_metrics = true;
    let source = value.source.clone();
    let output = value.to_xml().unwrap();
    assert!(output.contains("&amp;") && output.contains("&quot;"));
    let loaded = sensor(&output);
    compare_sensor(&sensor_projection(&loaded), &sensor_projection(&value));
    assert_eq!(value.source, source);
    assert_eq!(
        Sensor::default().to_xml().unwrap_err().code,
        ErrorCode::Unsupported
    );
    let mut mismatch = value.clone();
    mismatch.kind = "imu".into();
    assert_eq!(mismatch.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    let mut extension = value.clone();
    extension.gz_frame_id = Some("legacy".into());
    assert_eq!(extension.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    let plugin =
        sensor("<sensor name='s' type='force_torque'><plugin name='p' filename='f'/></sensor>");
    assert_eq!(plugin.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    let extension =
        sensor("<sensor name='s' type='force_torque'><alwaysOn>true</alwaysOn></sensor>");
    assert_eq!(extension.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    for bad in [
        " padded", "padded ", "a\tb", "a\nb", "a\rb", "a\0b", "\u{fffe}", "\u{ffff}",
    ] {
        for field in 0..4 {
            let mut v = value.clone();
            match field {
                0 => v.name = bad.into(),
                1 => v.pose.relative_to = bad.into(),
                2 => v.topic = bad.into(),
                _ => v.frame_id = bad.into(),
            };
            assert_eq!(v.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    for name in ["", "world", "__reserved__", "a::b"] {
        let mut v = value.clone();
        v.name = name.into();
        assert!(v.to_xml().is_err());
    }
    for topic in ["__default__", "a  b"] {
        let mut v = value.clone();
        v.topic = topic.into();
        assert!(v.to_xml().is_err());
    }
    for n in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        let mut v = value.clone();
        v.update_rate = n;
        assert!(v.to_xml().is_err());
        let mut v = value.clone();
        v.pose.transform.translation[0] = n;
        assert!(v.to_xml().is_err());
    }
    let mut v = value.clone();
    v.pose.transform.rotation = [0.; 4];
    assert!(v.to_xml().is_err());
    let mut v = value.clone();
    v.pose.transform.rotation[0] = f64::from_bits(1);
    assert!(v.to_xml().is_err());
    let mut v = value.clone();
    if let SensorConfig::ForceTorque(config) = &mut v.config {
        config.force[0].mean = f64::NAN;
    }
    assert!(v.to_xml().is_err());
}
#[test]
fn joint_sensor_payload_boundaries_preserve_old_admission() {
    let mut joint=load_joint_str("<joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='s' type='force_torque'/></joint>","joint.xml").unwrap();
    joint.sensors.push(joint.sensors[0].clone());
    assert_eq!(joint.to_xml().unwrap_err().code, ErrorCode::DuplicateName);
    for payload in [
        "<sensor name='s' type='imu'/>",
        "<sensor name='s' type='force_torque'><plugin name='p' filename='f'/></sensor>",
        "<physics/>",
    ] {
        let xml = format!(
            "<joint name='j' type='fixed'><parent>a</parent><child>b</child>{payload}</joint>"
        );
        assert_eq!(
            load_joint_str(&xml, "joint.xml").unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
}
