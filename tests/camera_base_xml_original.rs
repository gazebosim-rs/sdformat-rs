/*
 * Copyright (C) 2019 Open Source Robotics Foundation
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
// Generic Element/null identity stays unported; retained values map to owned fields.
use sdformat::*;
fn double(actual: f64, expected: f64) {
    assert!(
        actual.to_bits().abs_diff(expected.to_bits()) <= 4,
        "{actual} != {expected}"
    );
}
#[test]
fn original_camera_to_element_base_values_subset() {
    let mut camera = Camera {
        name: "my_camera".into(),
        horizontal_fov: 1.45,
        image_width: 123,
        image_height: 125,
        pixel_format: CameraPixelFormat::LInt8,
        near_clip: 0.2,
        far_clip: 200.2,
        visibility_mask: 123,
        save_frames: true,
        save_frames_path: "/tmp".into(),
        camera_info_topic: "/camera_info_test".into(),
        trigger_topic: "/trigger_topic_test".into(),
        triggered: true,
        ..Camera::default()
    };
    assert_eq!(camera.name, "my_camera");
    camera.pose.relative_to = "/frame".into();
    let result = camera.to_xml();
    assert!(result.is_ok());
    let xml = result.unwrap();
    assert!(!xml.is_empty());
    assert!(
        camera.source.is_none(),
        "supplemental owned construction provenance only"
    );
    let mut loaded = load_camera_str(&xml, "camera.xml").unwrap();
    double(loaded.horizontal_fov, 1.45);
    assert_eq!(loaded.image_width, 123);
    assert_eq!(loaded.image_height, 125);
    assert_eq!(loaded.pixel_format, CameraPixelFormat::LInt8);
    double(loaded.near_clip, 0.2);
    double(loaded.far_clip, 200.2);
    assert_eq!(loaded.visibility_mask, 123);
    assert_eq!(loaded.pose.relative_to, "/frame");
    assert!(loaded.save_frames);
    assert_eq!(loaded.save_frames_path, "/tmp");
    assert_eq!(loaded.camera_info_topic, "/camera_info_test");
    assert_eq!(loaded.trigger_topic, "/trigger_topic_test");
    assert!(loaded.triggered);
    loaded.near_clip = 0.33;
    let xml = loaded.to_xml().unwrap();
    assert!(!xml.is_empty());
    double(load_camera_str(&xml, "edited.xml").unwrap().near_clip, 0.33);
}
