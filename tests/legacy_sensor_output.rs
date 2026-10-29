// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[path = "support/legacy_sensor_projection.rs"]
mod legacy_sensor_projection;
use legacy_sensor_projection::{compare, projection, sensor};
fn rows() -> Vec<Value> {
    serde_json::from_str(include_str!("legacy_sensor_output.json")).unwrap()
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
    if let Some(dir) = std::env::var_os("SDF_LEGACY_SENSOR_OUTPUTS") {
        std::fs::write(std::path::Path::new(&dir).join(format!("{name}.xml")), xml).unwrap();
    }
}
#[test]
fn native_raw_kind_and_effective_classifier_roundtrip_separately() {
    for row in rows() {
        let value = sensor(row["xml"].as_str().unwrap());
        let expected = tokens(&row);
        compare(&projection(&value), &expected);
        if matches!(
            row["id"].as_str().unwrap(),
            "gps_old_boundary" | "ray_default" | "gpu_ray_default"
        ) {
            legacy_sensor_projection::assert_boundary(
                row["xml"].as_str().unwrap(),
                &value,
                row["id"].as_str().unwrap(),
            );
        }
        let output = value.to_xml().unwrap();
        compare(&projection(&sensor(&output)), &expected);
        assert_eq!(value.kind, row["kind"].as_str().unwrap());
        export(row["id"].as_str().unwrap(), &output);
    }
}
#[test]
fn typed_edits_preserve_raw_kind_and_input_provenance() {
    let rows = rows();
    for kind in ["gps", "ray", "gpu_ray"] {
        let old = rows
            .iter()
            .find(|r| r["id"] == format!("{kind}_legacy"))
            .unwrap();
        let expected = rows
            .iter()
            .find(|r| r["id"] == format!("{kind}_edited"))
            .unwrap();
        let mut value = sensor(old["xml"].as_str().unwrap());
        let source = value.source.clone();
        let raw = value.configuration_xml.clone();
        value.update_rate = 42.;
        match &mut value.config {
            SensorConfig::NavSat(v) => v.horizontal_position.mean = -7.5,
            SensorConfig::Lidar(v) => v.horizontal.samples = 111,
            _ => panic!("legacy config"),
        }
        let output = value.to_xml().unwrap();
        compare(&projection(&sensor(&output)), &tokens(expected));
        assert_eq!(value.kind, kind);
        assert_eq!(value.source, source);
        assert_eq!(value.configuration_xml, raw);
        export(&format!("{kind}-edited-authoritative"), &output);
    }
}
#[test]
fn advanced_payload_mismatch_and_joint_boundaries_are_unchanged() {
    for kind in ["gps", "ray", "gpu_ray"] {
        for extra in [
            "<plugin name='p' filename='f'/>",
            "<alwaysOn>true</alwaysOn>",
            "<gz_frame_id>legacy</gz_frame_id>",
        ] {
            let value = sensor(&format!("<sensor name='s' type='{kind}'>{extra}</sensor>"));
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
        }
        let input = format!(
            "<joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='s' type='{kind}'/></joint>"
        );
        assert_eq!(
            load_joint_str(&input, "joint.xml").unwrap_err().code,
            ErrorCode::Unsupported
        );
        let mut joint = load_joint_str(
            "<joint name='j' type='fixed'><parent>a</parent><child>b</child></joint>",
            "joint.xml",
        )
        .unwrap();
        joint
            .sensors
            .push(sensor(&format!("<sensor name='s' type='{kind}'/>")));
        assert_eq!(joint.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
    let mut value = sensor("<sensor name='s' type='gps'/>");
    value.kind = "ray".into();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    let mut value = sensor("<sensor name='s' type='ray'/>");
    let SensorConfig::Lidar(v) = &mut value.config else {
        panic!("lidar")
    };
    v.noise.precision = -0.;
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
}
