// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[path = "support/camera_projection.rs"]
mod camera_projection;
#[path = "support/camera_sensor_projection.rs"]
mod camera_sensor_projection;
use camera_projection::{compare, projection};
fn base() -> Camera {
    Camera {
        name: "base".into(),
        ..Camera::default()
    }
}
#[test]
fn native_base_camera_values_and_output_roundtrip() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("camera_base_xml.json")).unwrap();
    for row in rows {
        let expected: Vec<String> = row["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        let result = load_camera_str(row["xml"].as_str().unwrap(), "native.xml");
        if expected[0] == "REJECT" {
            assert_eq!(expected.len(), 1);
            assert!(result.is_err(), "{}", row["id"]);
            continue;
        }
        let camera = result.unwrap();
        compare(&projection(&camera), &expected);
        let output = camera.to_xml().unwrap();
        compare(
            &projection(&load_camera_str(&output, "rust.xml").unwrap()),
            &expected,
        );
        if row["id"] == "original" {
            compare(
                &projection(
                    &load_camera_str(
                        include_str!("fixtures/camera_to_element_native.xml"),
                        "fixture.xml",
                    )
                    .unwrap(),
                ),
                &expected,
            );
        }
        if let Some(directory) = std::env::var_os("SDF_CAMERA_BASE_XML_EXPORTS") {
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
fn advanced_retained_payload_is_rejected_instead_of_dropped() {
    let edits: [fn(&mut Camera); 20] = [
        |c| c.has_depth_near_clip = true,
        |c| c.has_depth_far_clip = true,
        |c| c.depth_near_clip = 0.2,
        |c| c.depth_far_clip = 20.,
        |c| c.has_segmentation_type = true,
        |c| c.segmentation_type = "panoptic".into(),
        |c| c.has_bounding_box_type = true,
        |c| c.bounding_box_type = "3d".into(),
        |c| c.distortion.k1 = 0.1,
        |c| c.distortion.center = [0.2, 0.4],
        |c| c.lens.kind = "custom".into(),
        |c| c.lens.intrinsics.fx = 2.,
        |c| c.lens.projection.tx = 2.,
        |c| c.lens.scale_to_hfov = false,
        |c| c.lens.cutoff_angle = 0.2,
        |c| c.image_noise.bias_mean = 1.,
        |c| c.image_noise.bias_stddev = 1.,
        |c| c.image_noise.dynamic_bias_stddev = 1.,
        |c| c.image_noise.dynamic_bias_correlation_time = 1.,
        |c| c.image_noise.precision = 1.,
    ];
    for edit in edits {
        let mut value = base();
        edit(&mut value);
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
}
#[test]
fn invalid_output_states_and_reader_diagnostics_remain_explicit() {
    assert!(Camera::default().to_xml().is_err());
    for field in 0..5 {
        for bad in [
            " padded", "padded ", "a  b", "a\tb", "a\nb", "a\rb", "a\0b", "\u{fffe}", "\u{ffff}",
        ] {
            let mut c = base();
            match field {
                0 => c.name = bad.into(),
                1 => c.pose.relative_to = bad.into(),
                2 => {
                    c.save_frames = true;
                    c.save_frames_path = bad.into();
                }
                3 => c.camera_info_topic = bad.into(),
                _ => c.trigger_topic = bad.into(),
            };
            assert!(c.to_xml().is_err());
        }
    }
    for n in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        for field in 0..5 {
            let mut c = base();
            match field {
                0 => c.horizontal_fov = n,
                1 => c.near_clip = n,
                2 => c.far_clip = n,
                3 => c.image_noise.mean = n,
                _ => c.image_noise.stddev = n,
            };
            assert!(c.to_xml().is_err());
        }
    }
    for field in 0..3 {
        let mut c = base();
        match field {
            0 => c.image_width = u32::MAX,
            1 => c.image_height = u32::MAX,
            _ => c.anti_aliasing = u32::MAX,
        };
        assert!(c.to_xml().is_err());
    }
    let edits: [fn(&mut Camera); 8] = [
        |c| c.horizontal_fov = 0.,
        |c| c.horizontal_fov = 7.,
        |c| c.near_clip = -1.,
        |c| c.far_clip = 0.,
        |c| c.image_noise.stddev = -1.,
        |c| c.save_frames = true,
        |c| c.save_frames_path = "/tmp/disabled".into(),
        |c| c.pose.transform.rotation = [0.; 4],
    ];
    for edit in edits {
        let mut c = base();
        edit(&mut c);
        assert!(c.to_xml().is_err());
    }
    for field in 0..2 {
        let mut c = base();
        if field == 0 {
            c.pose.transform.translation[0] = f64::NAN;
        } else {
            c.pose.transform.rotation[0] = f64::from_bits(1);
        }
        assert!(c.to_xml().is_err());
    }
    for xml in ["<", "<imu/>", "<camera/>"] {
        assert!(load_camera_str(xml, "bad.xml").is_err());
    }
    let error = load_camera_str(
        "\n<camera name='base'><horizontal_fov>0</horizontal_fov><image/><clip/></camera>",
        "caller.xml",
    )
    .unwrap_err();
    assert_eq!(error.source.uri, "caller.xml");
    assert_eq!(error.source.line, 2);
}
#[test]
fn constructor_values_and_full_document_presence_rules_are_preserved() {
    let value = base();
    let source = value.source.clone();
    let output = value.to_xml().unwrap();
    let loaded = load_camera_str(&output, "camera.xml").unwrap();
    compare(&projection(&loaded), &projection(&value));
    assert_eq!(value.source, source);
    assert!(loaded.lens.has_intrinsics && loaded.lens.has_projection);
    let rows: Vec<Value> = serde_json::from_str(include_str!("camera_base_xml.json")).unwrap();
    let row = rows.iter().find(|r| r["id"] == "minimal").unwrap();
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='camera'>{}</sensor></link></model></sdf>",
        row["xml"].as_str().unwrap()
    );
    let sdf = load_str(&xml, "model.sdf", &NoResources).unwrap();
    let SensorConfig::Camera(camera) = &sdf.models[0].links[0].sensors[0].config else {
        panic!("camera config")
    };
    assert_eq!(camera.name, "__default__");
    assert!(!camera.lens.has_intrinsics && !camera.lens.has_projection);
    let rows: Vec<Value> = serde_json::from_str(include_str!("camera_sensor_xml.json")).unwrap();
    let row = rows
        .iter()
        .find(|r| r["id"] == "full_document_original_boundary")
        .unwrap();
    assert_eq!(xml, row["xml"].as_str().unwrap());
    let expected: Vec<String> = row["tokens"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().into())
        .collect();
    let sensor = &sdf.models[0].links[0].sensors[0];
    camera_sensor_projection::compare(&camera_sensor_projection::projection(sensor), &expected);
    let output = sensor.to_xml();
    assert!(output.is_ok());
    let output = output.unwrap();
    assert!(!output.is_empty());
    let mut loaded = Sensor::default();
    loaded
        .load_element_str(&output, "camera-sensor.xml", &NoResources)
        .unwrap();
    camera_sensor_projection::compare(&camera_sensor_projection::projection(&loaded), &expected);
    if let Some(directory) = std::env::var_os("SDF_CAMERA_SENSOR_XML_EXPORTS") {
        std::fs::write(
            std::path::Path::new(&directory).join("full-document-authoritative.xml"),
            output,
        )
        .unwrap();
    }
}

#[test]
fn omitted_default_zero_payloads_reject_signed_bits() {
    let edits: [fn(&mut Camera); 14] = [
        |c| c.distortion.k1 = -0.,
        |c| c.distortion.k2 = -0.,
        |c| c.distortion.k3 = -0.,
        |c| c.distortion.p1 = -0.,
        |c| c.distortion.p2 = -0.,
        |c| c.lens.c3 = -0.,
        |c| {
            c.lens.has_intrinsics = false;
            c.lens.intrinsics.skew = -0.;
        },
        |c| {
            c.lens.has_projection = false;
            c.lens.projection.tx = -0.;
        },
        |c| {
            c.lens.has_projection = false;
            c.lens.projection.ty = -0.;
        },
        |c| c.image_noise.bias_mean = -0.,
        |c| c.image_noise.bias_stddev = -0.,
        |c| c.image_noise.dynamic_bias_stddev = -0.,
        |c| c.image_noise.dynamic_bias_correlation_time = -0.,
        |c| c.image_noise.precision = -0.,
    ];
    for edit in edits {
        let mut camera = base();
        edit(&mut camera);
        assert_eq!(camera.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
}
#[test]
fn emitted_zero_fields_preserve_their_signed_bits() {
    let mut camera = base();
    camera.near_clip = -0.;
    camera.image_noise.mean = -0.;
    camera.image_noise.stddev = -0.;
    camera.lens.intrinsics.skew = -0.;
    camera.lens.projection.tx = -0.;
    camera.lens.projection.ty = -0.;
    let xml = camera.to_xml().unwrap();
    let loaded = load_camera_str(&xml, "signed-zero.xml").unwrap();
    compare(&projection(&loaded), &projection(&camera));
}

#[test]
fn enabled_and_disabled_save_paths_validate_unknown_payloads() {
    for enabled in [false, true] {
        for path in [
            "<path unknown='x'>ignored</path>",
            "<path><unknown/></path>",
        ] {
            let camera =
                format!("<camera><image/><clip/><save enabled='{enabled}'>{path}</save></camera>");
            assert_eq!(
                load_camera_str(&camera, "camera.xml").unwrap_err().code,
                ErrorCode::Unsupported
            );
            let document = format!(
                "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='camera'>{camera}</sensor></link></model></sdf>"
            );
            assert_eq!(
                load_str(&document, "model.sdf", &NoResources)
                    .unwrap_err()
                    .code,
                ErrorCode::Unsupported
            );
        }
    }
}
