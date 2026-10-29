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
        .map(|n| format!("{n:02x}"))
        .collect()
}
fn projection(value: &AirSpeed) -> Vec<String> {
    let mut result = vec!["OK".into()];
    let noise = &value.pressure;
    result.push(
        match noise.kind {
            NoiseKind::None => "0",
            NoiseKind::Gaussian => "1",
            NoiseKind::GaussianQuantized => "2",
        }
        .into(),
    );
    result.extend(
        [
            noise.mean,
            noise.stddev,
            noise.bias_mean,
            noise.bias_stddev,
            noise.dynamic_bias_stddev,
            noise.dynamic_bias_correlation_time,
            noise.precision,
        ]
        .map(|n| n.to_bits().to_string()),
    );
    result
}

fn sensor(xml: &str) -> Sensor {
    let mut value = Sensor::default();
    value
        .load_element_str(xml, "sensor.xml", &NoResources)
        .unwrap();
    value
}
fn sensor_projection(value: &Sensor) -> Vec<String> {
    let mut result = vec![
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
    result.extend(
        value
            .pose
            .transform
            .translation
            .into_iter()
            .chain(value.pose.transform.rotation)
            .map(|n| n.to_bits().to_string()),
    );
    result.extend(projection(value.air_speed().unwrap()).into_iter().skip(1));
    result
}
fn compare_sensor(actual: &[String], expected: &[String]) {
    assert_eq!(actual.len(), 25);
    assert_eq!(expected.len(), 25);
    for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
        if (10..17).contains(&i) {
            let a = f64::from_bits(a.parse().unwrap());
            let b = f64::from_bits(b.parse().unwrap());
            assert!((a - b).abs() <= 1e-3, "pose {i}: {a} != {b}");
        } else {
            assert_eq!(a, b, "token {i}");
        }
    }
}
#[test]
fn native_values_and_actual_xml_roundtrips() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("air_speed_xml.json")).unwrap();
    for row in rows {
        let expected: Vec<String> = row["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        let input = row["xml"].as_str().unwrap();
        if expected[0] == "REJECT" {
            assert_eq!(expected.len(), 1);
            assert!(
                load_air_speed_str(input, "native.xml").is_err(),
                "{}",
                row["id"]
            );
            continue;
        }
        assert_eq!(expected[0], "OK");
        let output = if row["mode"] == "config" {
            assert_eq!(expected.len(), 9);
            let value = load_air_speed_str(input, "native.xml").unwrap();
            assert_eq!(projection(&value), expected, "{}", row["id"]);
            let output = value.to_xml().unwrap();
            assert_eq!(
                projection(&load_air_speed_str(&output, "rust.xml").unwrap()),
                expected
            );
            if row["id"] == "original" {
                let original = load_air_speed_str(
                    include_str!("fixtures/air_speed_to_element_native.xml"),
                    "fixture.xml",
                )
                .unwrap();
                assert_eq!(projection(&original), expected);
            }
            output
        } else {
            let value = sensor(input);
            compare_sensor(&sensor_projection(&value), &expected);
            let output = value.to_xml().unwrap();
            compare_sensor(&sensor_projection(&sensor(&output)), &expected);
            output
        };
        if let Some(directory) = std::env::var_os("SDF_AIR_SPEED_XML_EXPORTS") {
            std::fs::write(
                std::path::Path::new(&directory)
                    .join(format!("{}.xml", row["id"].as_str().unwrap())),
                output,
            )
            .unwrap();
        }
    }
}
#[test]
fn invalid_configurations_and_reader_locations_are_explicit() {
    for input in [
        "<",
        "<imu/>",
        "<air_speed unknown='x'/>",
        "<air_speed><unknown/></air_speed>",
        "<air_speed><pressure unknown='x'/></air_speed>",
        "<air_speed><pressure><unknown/></pressure></air_speed>",
    ] {
        assert!(load_air_speed_str(input, "bad.xml").is_err());
    }
    let error =
        load_air_speed_str("\n<air_speed><unknown/></air_speed>", "caller.xml").unwrap_err();
    assert_eq!(error.source.uri, "caller.xml");
    assert_eq!(error.source.line, 2);
    for bad in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        for field in 0..7 {
            let mut value = AirSpeed::default();
            let noise = &mut value.pressure;
            let fields = [
                &mut noise.mean,
                &mut noise.stddev,
                &mut noise.bias_mean,
                &mut noise.bias_stddev,
                &mut noise.dynamic_bias_stddev,
                &mut noise.dynamic_bias_correlation_time,
                &mut noise.precision,
            ];
            *fields.into_iter().nth(field).unwrap() = bad;
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    for field in 0..3 {
        let mut value = AirSpeed::default();
        match field {
            0 => value.pressure.stddev = -1.,
            1 => value.pressure.bias_stddev = -1.,
            _ => value.pressure.dynamic_bias_stddev = -1.,
        }
        assert!(value.to_xml().is_err());
    }
}
#[test]
fn matching_sensor_and_joint_payload_boundaries_are_explicit() {
    let mut value = sensor("<sensor name='s' type='air_speed'/>");
    value.kind = "imu".into();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    value.kind = "air_speed".into();
    if let SensorConfig::AirSpeed(config) = &mut value.config {
        config.pressure.mean = f64::NAN;
    }
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let input = "<joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='s' type='air_speed'/></joint>";
    assert_eq!(
        load_joint_str(input, "joint.xml").unwrap_err().code,
        ErrorCode::Unsupported
    );
    let mut joint = load_joint_str(
        "<joint name='j' type='fixed'><parent>a</parent><child>b</child></joint>",
        "joint.xml",
    )
    .unwrap();
    joint
        .sensors
        .push(sensor("<sensor name='s' type='air_speed'/>"));
    assert_eq!(joint.to_xml().unwrap_err().code, ErrorCode::Unsupported);
}
