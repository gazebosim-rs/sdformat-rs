// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

#[test]
fn native_sensor_frame_metadata() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("frame_id.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 30);
    for case in cases.as_array().unwrap() {
        let sdf = load_str(case["xml"].as_str().unwrap(), "frame.sdf", &NoResources).unwrap();
        let sensor = &sdf.models[0].links[0].sensors[0];
        assert_eq!(
            sensor.frame_id,
            case["frame_id"].as_str().unwrap(),
            "{case}"
        );
        assert_eq!(
            sensor.gz_frame_id.as_deref(),
            case["gz_frame_id"].as_str(),
            "{case}"
        );
    }
}

#[test]
fn frame_metadata_keeps_unknown_extensions_and_nested_content_unsupported() {
    for content in [
        "<other:frame_id>id</other:frame_id>",
        "<gz:frame_id>id</gz:frame_id>",
        "<gz_other_frame_id>id</gz_other_frame_id>",
        "<frame_id value='id'/>",
        "<gz_frame_id value='id'/>",
        "<frame_id><nested/></frame_id>",
        "<gz_frame_id><nested/></gz_frame_id>",
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='imu'>{content}</sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "frame.sdf", &NoResources).unwrap_err().code,
            ErrorCode::Unsupported,
            "{content}"
        );
    }
    for tag in ["frame_id", "gz_frame_id"] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='imu'><{tag}/><{tag}/></sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "frame.sdf", &NoResources).unwrap_err().code,
            ErrorCode::InvalidValue
        );
    }
}
