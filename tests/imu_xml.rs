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
fn projection(imu: &Imu) -> Vec<String> {
    let mut result = vec![
        "OK".into(),
        u8::from(imu.enable_orientation).to_string(),
        hex(&imu.localization),
        hex(&imu.custom_rpy_parent_frame),
        hex(&imu.gravity_dir_x_parent_frame),
    ];
    result.extend(
        imu.custom_rpy
            .into_iter()
            .chain(imu.gravity_dir_x)
            .map(|n| n.to_bits().to_string()),
    );
    for noise in imu.angular_velocity.iter().chain(&imu.linear_acceleration) {
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
    result.extend(projection(value.imu().unwrap()).into_iter().skip(1));
    result
}
fn compare_sensor(actual: &[String], expected: &[String]) {
    assert_eq!(actual.len(), 75);
    assert_eq!(expected.len(), 75);
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
fn native_configuration_and_sensor_values_roundtrip() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("imu_xml.json")).unwrap();
    for row in rows {
        let expected: Vec<String> = row["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        assert_eq!(expected[0], "OK");
        let xml = if row["mode"] == "config" {
            assert_eq!(expected.len(), 59);
            let value = load_imu_str(row["xml"].as_str().unwrap(), "native.xml").unwrap();
            assert_eq!(projection(&value), expected, "{}", row["id"]);
            let xml = value.to_xml().unwrap();
            assert_eq!(
                projection(&load_imu_str(&xml, "rust.xml").unwrap()),
                expected,
                "{}",
                row["id"]
            );
            if row["id"] == "original" {
                assert_eq!(
                    projection(
                        &load_imu_str(
                            include_str!("fixtures/imu_to_element_native.xml"),
                            "fixture.xml"
                        )
                        .unwrap()
                    ),
                    expected
                );
                if let Some(path) = std::env::var_os("SDF_IMU_XML_EXPORT") {
                    std::fs::write(path, &xml).unwrap();
                }
            }
            xml
        } else {
            let value = sensor(row["xml"].as_str().unwrap());
            compare_sensor(&sensor_projection(&value), &expected);
            let xml = value.to_xml().unwrap();
            compare_sensor(&sensor_projection(&sensor(&xml)), &expected);
            xml
        };
        if row["mode"] == "sensor"
            && let Some(directory) = std::env::var_os("SDF_IMU_XML_EXPORTS")
        {
            std::fs::write(
                std::path::Path::new(&directory)
                    .join(format!("{}.xml", row["id"].as_str().unwrap())),
                xml,
            )
            .unwrap();
        }
    }
}
#[test]
fn raw_vectors_and_edited_values_are_not_normalized() {
    let mut value = Imu {
        gravity_dir_x: [0.; 3],
        custom_rpy: [1., 2., 3.],
        enable_orientation: false,
        localization: "NED".into(),
        ..Imu::default()
    };
    let output = value.to_xml().unwrap();
    let loaded = load_imu_str(&output, "imu.xml").unwrap();
    assert_eq!(loaded.gravity_dir_x, [0.; 3]);
    assert_eq!(loaded.custom_rpy, [1., 2., 3.]);
    assert_eq!(loaded, value);
    value.gravity_dir_x = [1., 2., 3.];
    assert_eq!(
        load_imu_str(&value.to_xml().unwrap(), "edited.xml").unwrap(),
        value
    );
    let sensor = sensor("<sensor name='s' type='imu'/>");
    assert!(sensor.to_xml().is_ok());
}
#[test]
fn unreadable_states_and_reader_errors_are_explicit() {
    for xml in ["<", "<force_torque/>", "<imu><unknown/></imu>"] {
        assert!(load_imu_str(xml, "bad.xml").is_err());
    }
    for field in 0..3 {
        for bad in [
            " padded", "padded ", "a\tb", "a\nb", "a\rb", "a\0b", "\u{fffe}", "\u{ffff}",
        ] {
            let mut value = Imu::default();
            match field {
                0 => value.localization = bad.into(),
                1 => value.custom_rpy_parent_frame = bad.into(),
                _ => value.gravity_dir_x_parent_frame = bad.into(),
            };
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    for text in ["", "a  b"] {
        let value = Imu {
            localization: text.into(),
            ..Imu::default()
        };
        assert!(value.to_xml().is_err());
    }
    for n in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        for field in 0..2 {
            let mut value = Imu::default();
            if field == 0 {
                value.custom_rpy[0] = n;
            } else {
                value.gravity_dir_x[0] = n;
            }
            assert!(value.to_xml().is_err());
        }
    }
    let mut value = Imu::default();
    value.angular_velocity[0].stddev = -1.;
    assert!(value.to_xml().is_err());
    let error = load_imu_str("\n<imu><unknown/></imu>", "caller.xml").unwrap_err();
    assert_eq!(error.source.uri, "caller.xml");
    assert_eq!(error.source.line, 2);
}
#[test]
fn sensor_family_and_joint_boundaries_remain_explicit() {
    let mut value = sensor("<sensor name='s' type='imu'/>");
    value.update_rate = -1.;
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    value.update_rate = 1.;
    value.kind = "force_torque".into();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    value.kind = "imu".into();
    if let SensorConfig::Imu(imu) = &mut value.config {
        imu.gravity_dir_x[0] = f64::NAN;
    }
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    for xml in [
        "<sensor name='s' type='imu'><plugin name='p' filename='f'/></sensor>",
        "<sensor name='s' type='imu'><alwaysOn>true</alwaysOn></sensor>",
        "<sensor name='s' type='imu'><gz_frame_id>legacy</gz_frame_id></sensor>",
    ] {
        assert_eq!(
            sensor(xml).to_xml().unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
    let xml = "<joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='s' type='imu'/></joint>";
    assert_eq!(
        load_joint_str(xml, "joint.xml").unwrap_err().code,
        ErrorCode::Unsupported
    );
}
