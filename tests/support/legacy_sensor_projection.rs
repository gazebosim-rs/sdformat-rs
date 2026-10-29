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
fn number(output: &mut Vec<String>, value: f64) {
    output.push(value.to_bits().to_string());
}
fn noise(output: &mut Vec<String>, value: &Noise) {
    output.push(
        match value.kind {
            NoiseKind::None => "0",
            NoiseKind::Gaussian => "1",
            NoiseKind::GaussianQuantized => "2",
        }
        .into(),
    );
    output.extend(
        [
            value.mean,
            value.stddev,
            value.bias_mean,
            value.bias_stddev,
            value.dynamic_bias_stddev,
            value.dynamic_bias_correlation_time,
            value.precision,
        ]
        .map(|v| v.to_bits().to_string()),
    );
}
pub fn projection(value: &Sensor) -> Vec<String> {
    let mut output = vec![
        "OK".into(),
        hex(&value.name),
        hex(&value.kind),
        hex(value.effective_type().unwrap().as_str()),
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
    match &value.config {
        SensorConfig::NavSat(v) => {
            for n in [
                &v.horizontal_position,
                &v.vertical_position,
                &v.horizontal_velocity,
                &v.vertical_velocity,
            ] {
                noise(&mut output, n);
            }
        }
        SensorConfig::Lidar(v) => {
            for scan in [&v.horizontal, &v.vertical] {
                output.push(scan.samples.to_string());
                for n in [scan.resolution, scan.min_angle, scan.max_angle] {
                    number(&mut output, n);
                }
            }
            for n in [v.range_min, v.range_max, v.range_resolution] {
                number(&mut output, n);
            }
            output.push(v.visibility_mask.to_string());
            noise(&mut output, &v.noise);
        }
        _ => panic!("legacy navigation/ray configuration expected"),
    }
    output
}
pub fn compare(actual: &[String], expected: &[String]) {
    assert!(matches!(expected.len(), 38 | 50));
    assert_eq!(actual.len(), expected.len());
    for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
        if (11..18).contains(&i) {
            let a = f64::from_bits(a.parse().unwrap());
            let b = f64::from_bits(b.parse().unwrap());
            assert!((a - b).abs() <= 1e-3, "pose {i}: {a} != {b}");
        } else {
            assert_eq!(a, b, "token {i}");
        }
    }
}
pub fn sensor(xml: &str) -> Sensor {
    let mut value = Sensor::default();
    value
        .load_element_str(xml, "legacy.xml", &NoResources)
        .unwrap();
    value
}
pub fn assert_boundary(input: &str, value: &Sensor, id: &str) {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("../legacy_sensor_output.json")).unwrap();
    let row = rows.iter().find(|r| r["id"] == id).unwrap();
    assert_eq!(row["xml"], input);
    let expected: Vec<String> = row["tokens"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().into())
        .collect();
    compare(&projection(value), &expected);
    let output = value.to_xml();
    assert!(output.is_ok());
    let output = output.unwrap();
    assert!(!output.is_empty());
    compare(&projection(&sensor(&output)), &expected);
    if let Some(directory) = std::env::var_os("SDF_LEGACY_SENSOR_OUTPUTS") {
        std::fs::write(
            std::path::Path::new(&directory)
                .join(format!("{}-boundary-authoritative.xml", value.kind)),
            output,
        )
        .unwrap();
    }
}
