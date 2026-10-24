// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

#[test]
fn native_sensor_topic_text() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("topic.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 24);
    for case in cases.as_array().unwrap() {
        let sdf = load_str(case["xml"].as_str().unwrap(), "topic.sdf", &NoResources).unwrap();
        assert_eq!(
            sdf.models[0].links[0].sensors[0].topic,
            case["topic"].as_str().unwrap(),
            "{case}"
        );
    }
}

#[test]
fn sensor_topic_is_a_supported_leaf_only() {
    for content in ["<topic name='t'/>", "<topic><child/></topic>"] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='imu'>{content}</sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "topic.sdf", &NoResources).unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
}
