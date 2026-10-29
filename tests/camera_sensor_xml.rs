// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "support/camera_projection.rs"]
mod camera_projection;
use sdformat::*;
use serde_json::Value;

fn sensor(xml: &str) -> Result<Sensor, Diagnostic> {
    let mut value = Sensor::default();
    value.load_element_str(xml, "sensor.xml", &NoResources)?;
    Ok(value)
}
#[path = "support/camera_sensor_projection.rs"]
mod camera_sensor_projection;
use camera_sensor_projection::{compare, projection};
fn rows() -> Vec<Value> {
    serde_json::from_str(include_str!("camera_sensor_xml.json")).unwrap()
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
    if let Some(directory) = std::env::var_os("SDF_CAMERA_SENSOR_XML_EXPORTS") {
        std::fs::write(
            std::path::Path::new(&directory).join(format!("{name}.xml")),
            xml,
        )
        .unwrap();
    }
}
#[test]
fn native_envelope_and_camera_values_roundtrip() {
    for row in rows() {
        let expected = tokens(&row);
        let result = if row["mode"] == "document" {
            load_str(row["xml"].as_str().unwrap(), "model.sdf", &NoResources)
                .map(|sdf| sdf.models[0].links[0].sensors[0].clone())
        } else {
            sensor(row["xml"].as_str().unwrap())
        };
        if expected[0] == "REJECT" {
            assert_eq!(expected.len(), 1);
            assert_eq!(row["rust_read"], "ok_generic_metadata");
            assert_eq!(row["rust_output_error"], "InvalidValue");
            let value = result.expect("existing generic Sensor numeric admission");
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
            continue;
        }
        let value = result.unwrap();
        compare(&projection(&value), &expected);
        let xml = value.to_xml().unwrap();
        compare(&projection(&sensor(&xml).unwrap()), &expected);
        export(row["id"].as_str().unwrap(), &xml);
    }
}
#[test]
fn typed_edits_are_authoritative_without_mutating_provenance() {
    let rows = rows();
    let original = rows
        .iter()
        .find(|r| r["id"] == "two_poses_and_escaped_fields")
        .unwrap();
    let expected = rows.iter().find(|r| r["id"] == "typed_edited").unwrap();
    let mut value = sensor(original["xml"].as_str().unwrap()).unwrap();
    let source = value.source.clone();
    let configuration = value.configuration_xml.clone();
    let camera_source = value.camera().unwrap().source.clone();
    value.update_rate = 42.;
    value.pose.transform.translation[0] = 9.5;
    let SensorConfig::Camera(camera) = &mut value.config else {
        panic!("camera configuration")
    };
    camera.image_width = 96;
    camera.camera_info_topic = "/edited&info".into();
    camera.pose.transform.translation[1] = 8.25;
    let xml = value.to_xml().unwrap();
    compare(&projection(&sensor(&xml).unwrap()), &tokens(expected));
    assert_eq!(value.source, source);
    assert_eq!(value.configuration_xml, configuration);
    assert_eq!(value.camera().unwrap().source, camera_source);
    export("edited-authoritative", &xml);
}
#[test]
fn existing_camera_guards_and_constructor_values_are_preserved() {
    let unnamed = sensor("<sensor name='s' type='camera'/>").unwrap();
    assert!(unnamed.camera().unwrap().name.is_empty());
    assert_eq!(unnamed.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let edits: [fn(&mut Camera); 14] = [
        |c| c.distortion.k1 = -0.,
        |c| c.distortion.k2 = -0.,
        |c| c.distortion.k3 = -0.,
        |c| c.distortion.p1 = -0.,
        |c| c.distortion.p2 = -0.,
        |c| c.lens.c3 = -0.,
        |c| c.lens.intrinsics.skew = -0.,
        |c| c.lens.projection.tx = -0.,
        |c| c.lens.projection.ty = -0.,
        |c| c.image_noise.bias_mean = -0.,
        |c| c.image_noise.bias_stddev = -0.,
        |c| c.image_noise.dynamic_bias_stddev = -0.,
        |c| c.image_noise.dynamic_bias_correlation_time = -0.,
        |c| c.image_noise.precision = -0.,
    ];
    for edit in edits {
        let mut value = sensor(
            "<sensor name='s' type='camera'><camera name='c'><image/><clip/></camera></sensor>",
        )
        .unwrap();
        let SensorConfig::Camera(camera) = &mut value.config else {
            panic!("camera configuration")
        };
        edit(camera);
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
    let mut value =
        sensor("<sensor name='s' type='camera'><camera name='c'><image/><clip/></camera></sensor>")
            .unwrap();
    let SensorConfig::Camera(camera) = &mut value.config else {
        panic!("camera configuration")
    };
    camera.horizontal_fov = f64::NAN;
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
}
#[test]
fn unsupported_sensor_payloads_kinds_and_joint_admission_stay_explicit() {
    for kind in ["depth_camera", "thermal_camera", "rgbd_camera"] {
        let value = sensor(&format!(
            "<sensor name='s' type='{kind}'><camera name='c'><image/><clip/></camera></sensor>"
        ))
        .unwrap();
        assert_eq!(value.kind, kind);
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
    for extra in [
        "<plugin name='p' filename='f'/>",
        "<alwaysOn>true</alwaysOn>",
        "<gz_frame_id>legacy</gz_frame_id>",
    ] {
        let value=sensor(&format!("<sensor name='s' type='camera'><camera name='c'><image/><clip/></camera>{extra}</sensor>")).unwrap();
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
    let input = "<joint name='j' type='fixed'><parent>a</parent><child>b</child><sensor name='s' type='camera'><camera name='c'><image/><clip/></camera></sensor></joint>";
    assert_eq!(
        load_joint_str(input, "joint.xml").unwrap_err().code,
        ErrorCode::Unsupported
    );
    let mut joint = load_joint_str(
        "<joint name='j' type='fixed'><parent>a</parent><child>b</child></joint>",
        "joint.xml",
    )
    .unwrap();
    joint.sensors.push(
        sensor("<sensor name='s' type='camera'><camera name='c'><image/><clip/></camera></sensor>")
            .unwrap(),
    );
    assert_eq!(joint.to_xml().unwrap_err().code, ErrorCode::Unsupported);
}
