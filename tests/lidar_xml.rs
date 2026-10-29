// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "support/legacy_sensor_projection.rs"]
mod legacy_sensor_projection;
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
fn projection(value: &Lidar) -> Vec<String> {
    let mut result = vec!["OK".into()];
    for scan in [&value.horizontal, &value.vertical] {
        result.push(scan.samples.to_string());
        result.extend(
            [scan.resolution, scan.min_angle, scan.max_angle].map(|n| n.to_bits().to_string()),
        );
    }
    result.extend(
        [value.range_min, value.range_max, value.range_resolution].map(|n| n.to_bits().to_string()),
    );
    result.push(value.visibility_mask.to_string());
    result.push(
        match value.noise.kind {
            NoiseKind::None => "0",
            NoiseKind::Gaussian => "1",
            NoiseKind::GaussianQuantized => "2",
        }
        .into(),
    );
    result.extend(
        [
            value.noise.mean,
            value.noise.stddev,
            value.noise.bias_mean,
            value.noise.bias_stddev,
            value.noise.dynamic_bias_stddev,
            value.noise.dynamic_bias_correlation_time,
            value.noise.precision,
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
    result.extend(projection(value.lidar().unwrap()).into_iter().skip(1));
    result
}
fn compare_sensor(actual: &[String], expected: &[String]) {
    assert_eq!(actual.len(), 37);
    assert_eq!(expected.len(), 37);
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
    let rows: Vec<Value> = serde_json::from_str(include_str!("lidar_xml.json")).unwrap();
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
                load_lidar_str(input, "native.xml").is_err(),
                "{}",
                row["id"]
            );
            continue;
        }
        assert_eq!(expected[0], "OK");
        let output = if row["mode"] == "config" {
            assert_eq!(expected.len(), 21);
            let value = load_lidar_str(input, "native.xml").unwrap();
            assert_eq!(projection(&value), expected, "{}", row["id"]);
            let output = value.to_xml().unwrap();
            assert_eq!(
                projection(&load_lidar_str(&output, "rust.xml").unwrap()),
                expected
            );
            if row["id"] == "original" {
                let original = load_lidar_str(
                    include_str!("fixtures/lidar_to_element_native.xml"),
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
        if let Some(directory) = std::env::var_os("SDF_LIDAR_XML_EXPORTS") {
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
fn invalid_numbers_and_reader_locations_are_explicit() {
    for input in [
        "<",
        "<imu/>",
        "<lidar unknown='x'/>",
        "<lidar><unknown/></lidar>",
        "<lidar/>",
        "<lidar><scan/><range/></lidar>",
        "<lidar><scan><horizontal/></scan></lidar>",
        "<lidar><scan><horizontal><unknown/></horizontal></scan><range/></lidar>",
    ] {
        assert!(load_lidar_str(input, "bad.xml").is_err());
    }
    let error = load_lidar_str("\n<lidar><unknown/></lidar>", "caller.xml").unwrap_err();
    assert_eq!(error.source.uri, "caller.xml");
    assert_eq!(error.source.line, 2);
    for field in 0..11 {
        for bad in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
            let mut value = Lidar::default();
            let fields = [
                &mut value.horizontal.resolution,
                &mut value.horizontal.min_angle,
                &mut value.horizontal.max_angle,
                &mut value.vertical.resolution,
                &mut value.vertical.min_angle,
                &mut value.vertical.max_angle,
                &mut value.range_min,
                &mut value.range_max,
                &mut value.range_resolution,
                &mut value.noise.mean,
                &mut value.noise.stddev,
            ];
            *fields.into_iter().nth(field).unwrap() = bad;
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    let mut value = Lidar::default();
    value.noise.stddev = -1.;
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
}
#[test]
fn omitted_noise_fields_are_never_silently_discarded() {
    for field in 0..5 {
        for bad in [-0., 0.1, -0.1, f64::NAN, f64::INFINITY] {
            let mut value = Lidar::default();
            let fields = [
                &mut value.noise.bias_mean,
                &mut value.noise.bias_stddev,
                &mut value.noise.dynamic_bias_stddev,
                &mut value.noise.dynamic_bias_correlation_time,
                &mut value.noise.precision,
            ];
            *fields.into_iter().nth(field).unwrap() = bad;
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
        }
    }
}
#[test]
fn matching_sensor_and_joint_payload_boundaries_are_explicit() {
    for kind in ["ray", "gpu_ray"] {
        let input = format!("<sensor name='s' type='{kind}'/>");
        let legacy = sensor(&input);
        assert_eq!(legacy.kind, kind);
        legacy_sensor_projection::assert_boundary(&input, &legacy, &format!("{kind}_default"));
    }
    for root in ["ray", "gpu_ray", "gpu_lidar"] {
        assert!(load_lidar_str(&format!("<{root}/>"), "legacy.xml").is_err());
    }

    let mut value = sensor("<sensor name='s' type='lidar'/>");
    value.kind = "imu".into();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    value.kind = "lidar".into();
    if let SensorConfig::Lidar(config) = &mut value.config {
        config.noise.mean = f64::NAN;
    }
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let input = "<joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='s' type='lidar'/></joint>";
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
        .push(sensor("<sensor name='s' type='lidar'/>"));
    assert_eq!(joint.to_xml().unwrap_err().code, ErrorCode::Unsupported);
}
