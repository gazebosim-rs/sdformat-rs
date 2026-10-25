/*
 * Copyright 2018 Open Source Robotics Foundation
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

use sdformat::{Camera, ErrorCode, NoResources, SensorConfig, SensorType, load_str};

fn fragment(fixture: &str, name: &str, kind: &str) -> String {
    let tag = format!("<sensor name=\"{name}\" type=\"{kind}\">");
    let start = fixture.find(&tag).unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{}</link></model></sdf>",
        &fixture[start..end]
    )
}

#[test]
fn native_link_segmentation_and_bounding_box_camera_fragments_are_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    for (name, kind, effective, camera_name) in [
        (
            "segmentation_sensor",
            "segmentation",
            SensorType::SegmentationCamera,
            "my_segmentation_camera",
        ),
        (
            "boundingbox_sensor",
            "boundingbox",
            SensorType::BoundingBoxCamera,
            "my_boundingbox_camera",
        ),
    ] {
        let xml = fragment(fixture, name, kind);
        let sdf = load_str(&xml, "sensors.sdf#segmentation-box", &NoResources).unwrap();
        let sensor = &sdf.models[0].links[0].sensors[0];
        assert_eq!(sensor.name, name);
        assert_eq!(sensor.kind, kind);
        assert_eq!(sensor.effective_type(), Some(effective));
        assert!(matches!(sensor.config, SensorConfig::Camera(_)));
        assert_eq!(sensor.pose.transform.translation, [37., 38., 39.]);
        assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
        let camera = sensor.camera().unwrap();
        assert_eq!(camera.name, camera_name);
        if kind == "segmentation" {
            assert!(camera.has_segmentation_type);
            assert_eq!(camera.segmentation_type, "semantic");
            assert!(!camera.has_bounding_box_type);
        } else {
            assert!(camera.has_bounding_box_type);
            assert_eq!(camera.bounding_box_type, "2d");
            assert!(!camera.has_segmentation_type);
        }
        assert_eq!(camera.pose.transform.translation, [0.1, 0.2, 0.3]);
        assert_eq!(camera.pose.transform.rotation, [0., 0., 0., 1.]);
        assert_eq!(camera.image_width, 640);
        assert_eq!(camera.image_height, 480);
        assert_eq!(camera.near_clip, 0.2);
        assert_eq!(camera.far_clip, 12.3);
        assert!(camera.source.is_some());
        assert!(sensor.configuration_xml.starts_with("<camera name="));
    }
}

#[test]
fn camera_family_presence_and_alias_controls_are_explicit() {
    for (kind, effective) in [
        ("segmentation", SensorType::SegmentationCamera),
        ("segmentation_camera", SensorType::SegmentationCamera),
        ("boundingbox", SensorType::BoundingBoxCamera),
        ("boundingbox_camera", SensorType::BoundingBoxCamera),
    ] {
        let xml = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'/></link></model></sdf>"
        );
        let sdf = load_str(&xml, "camera-family-alias.sdf", &NoResources).unwrap();
        let sensor = &sdf.models[0].links[0].sensors[0];
        assert_eq!(sensor.kind, kind);
        assert_eq!(sensor.effective_type(), Some(effective));
        assert_eq!(sensor.camera(), Some(&Camera::default()));
        assert!(sensor.configuration_xml.is_empty());

        let mismatch = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'><imu/></sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&mismatch, "bad-camera-family.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}

#[test]
fn explicit_camera_type_flags_preserve_presence() {
    for child in [
        "<segmentation_type>semantic</segmentation_type>",
        "<box_type>2d</box_type>",
    ] {
        let xml = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='camera'><camera><image/><clip/>{child}</camera></sensor></link></model></sdf>"
        );
        let sdf = load_str(&xml, "camera-type-flag.sdf", &NoResources).unwrap();
        let camera = sdf.models[0].links[0].sensors[0].camera().unwrap();
        assert_eq!(
            camera.has_segmentation_type,
            child.contains("segmentation_type")
        );
        assert_eq!(camera.has_bounding_box_type, child.contains("box_type"));
    }
}
