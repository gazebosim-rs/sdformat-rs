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
fn native_link_depth_and_rgbd_camera_fragments_are_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    for (name, kind, effective, translation, metrics, camera_name, triggered) in [
        (
            "depth_sensor",
            "depth",
            SensorType::DepthCamera,
            [7., 8., 9.],
            true,
            "my_depth_camera",
            false,
        ),
        (
            "rgbd_sensor",
            "rgbd",
            SensorType::RgbdCamera,
            [37., 38., 39.],
            false,
            "my_rgbd_camera",
            true,
        ),
    ] {
        let xml = fragment(fixture, name, kind);
        let sdf = load_str(&xml, "sensors.sdf#depth-rgbd", &NoResources).unwrap();
        let sensor = &sdf.models[0].links[0].sensors[0];
        assert_eq!(sensor.name, name);
        assert_eq!(sensor.kind, kind);
        assert_eq!(sensor.effective_type(), Some(effective));
        assert!(matches!(sensor.config, SensorConfig::Camera(_)));
        assert_eq!(sensor.pose.transform.translation, translation);
        assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
        assert_eq!(sensor.enable_metrics, metrics);
        let camera = sensor.camera().unwrap();
        assert_eq!(camera.name, camera_name);
        assert_eq!(camera.triggered, triggered);
        assert_eq!(sensor.topic, "");
        assert_eq!(camera.trigger_topic, "");
        assert_eq!(camera.pose.transform.translation, [0.1, 0.2, 0.3]);
        assert_eq!(camera.pose.transform.rotation, [0., 0., 0., 1.]);
        assert_eq!(camera.horizontal_fov, 0.75);
        assert_eq!(camera.image_width, 640);
        assert_eq!(camera.image_height, 480);
        assert_eq!(camera.near_clip, 0.2);
        assert_eq!(camera.far_clip, 12.3);
        assert!(camera.source.is_some());
        assert!(sensor.configuration_xml.starts_with("<camera name="));
    }
}

#[test]
fn native_camera_family_aliases_and_mismatched_children_are_bounded() {
    for (kind, effective) in [
        ("depth", SensorType::DepthCamera),
        ("depth_camera", SensorType::DepthCamera),
        ("rgbd", SensorType::RgbdCamera),
        ("rgbd_camera", SensorType::RgbdCamera),
    ] {
        let xml = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'/></link></model></sdf>"
        );
        let sdf = load_str(&xml, "camera-alias.sdf", &NoResources).unwrap();
        let sensor = &sdf.models[0].links[0].sensors[0];
        assert_eq!(sensor.kind, kind);
        assert_eq!(sensor.effective_type(), Some(effective));
        assert_eq!(sensor.camera(), Some(&Camera::default()));
        assert!(sensor.configuration_xml.is_empty());

        let mismatch = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'><imu/></sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&mismatch, "bad-camera-alias.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}
