/*
 * Copyright (C) 2018 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Fixed value observations from DOMFrame.ToElement and ToElementErrorOutput.
#[path = "support/quiet.rs"]
mod quiet;
use glam::{DQuat, EulerRot};
use sdformat::{ErrorCode, Frame, Pose, Source, load_frame_str};

fn source_frame(with_attachment: bool) -> Frame {
    let mut frame = Frame {
        name: "my-frame".into(),
        attached_to: String::new(),
        pose: Pose::default(),
        source: Source {
            uri: "frame-roundtrip".into(),
            line: 0,
            column: 0,
        },
    };
    frame.pose.transform.translation = [1., 2., 3.];
    frame.pose.transform.rotation = DQuat::from_euler(EulerRot::ZYX, 0.3, 0.2, 0.1).to_array();
    if with_attachment {
        frame.attached_to = "attached-to-frame".into();
        frame.pose.relative_to = "relative-to-frame".into();
    }
    frame
}

fn check_roundtrip(with_attachment: bool) {
    let frame = source_frame(with_attachment);
    // The pinned original compares frame.Name() to itself, not frame2.Name().
    assert_eq!(frame.name, "my-frame");
    if !with_attachment {
        assert!(frame.attached_to.is_empty());
    }
    let output = frame.to_xml();
    assert!(output.is_ok());
    let xml = output.expect("bounded Frame output");
    assert!(
        !xml.is_empty(),
        "successful XML maps the non-null emitted Element"
    );
    let loaded = load_frame_str(&xml, "frame-roundtrip").expect("bounded Frame load");
    assert!(xml.starts_with("<frame "));
    assert_eq!(loaded.name, "my-frame"); // Supplemental; native name check is tautological.
    assert_eq!(loaded.attached_to, frame.attached_to);
    assert_eq!(loaded.pose.relative_to, frame.pose.relative_to);
    assert_eq!(loaded.pose.transform.translation, [1., 2., 3.]);
    let expected = DQuat::from_array(frame.pose.transform.rotation);
    let actual = DQuat::from_array(loaded.pose.transform.rotation);
    assert!((actual.dot(expected).abs() - 1.).abs() < 1e-12);
    // Native Pose3 equality observes quaternion signs component by component.
    for (actual, expected) in loaded
        .pose
        .transform
        .rotation
        .into_iter()
        .zip(frame.pose.transform.rotation)
    {
        assert!((actual - expected).abs() <= 1e-3);
    }
}

#[test]
fn partial_dom_frame_to_element_values() {
    check_roundtrip(true);
    check_roundtrip(false);
}

#[test]
fn partial_dom_frame_to_element_error_output_values() {
    quiet::assert_quiet(
        "partial_dom_frame_to_element_error_output_values",
        "SDF_FRAME_XML_QUIET_CHILD",
        || {
            check_roundtrip(true);
            check_roundtrip(false);
        },
    );
}

#[test]
fn supplemental_frame_xml_boundaries() {
    let mut frame = source_frame(false);
    frame.name = "a & \"quoted\"".into();
    let xml = frame.to_xml().unwrap();
    assert!(xml.contains("&amp;"));
    assert!(xml.contains("&quot;"));
    assert_eq!(load_frame_str(&xml, "test").unwrap().name, frame.name);

    frame.name.clear();
    assert_eq!(frame.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    for name in [" world", "world", "__bad__", "a::b", "a\nb"] {
        frame.name = name.into();
        assert_eq!(frame.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    frame.name = "valid".into();
    frame.attached_to = "a\tb".into();
    assert_eq!(frame.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    frame.attached_to.clear();
    frame.pose.relative_to = "a\nb".into();
    assert_eq!(frame.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    frame.pose.relative_to.clear();
    frame.pose.transform.rotation = [0.; 4];
    assert_eq!(frame.to_xml().unwrap_err().code, ErrorCode::InvalidValue);

    assert_eq!(
        load_frame_str("<world/>", "test").unwrap_err().code,
        ErrorCode::Xml
    );
    assert!(load_frame_str("<frame", "test").is_err());
    assert_eq!(
        load_frame_str("<frame/>", "test").unwrap_err().code,
        ErrorCode::Missing
    );
}
