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

use sdformat::{
    Camera, CameraPixelFormat, ErrorCode, NoResources, SensorConfig, SensorType, load_str,
};

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
fn native_link_thermal_camera_fragment_is_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let xml = fragment(fixture, "thermal_sensor", "thermal");
    let sdf = load_str(&xml, "sensors.sdf#thermal", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "thermal_sensor");
    assert_eq!(sensor.kind, "thermal");
    assert_eq!(sensor.effective_type(), Some(SensorType::ThermalCamera));
    assert!(matches!(sensor.config, SensorConfig::Camera(_)));
    assert_eq!(sensor.pose.transform.translation, [37., 38., 39.]);
    assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(!sensor.enable_metrics);
    let camera = sensor.camera().unwrap();
    assert_eq!(camera.name, "my_thermal_camera");
    assert_eq!(camera.pose.transform.translation, [0.1, 0.2, 0.3]);
    assert_eq!(camera.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(camera.source.is_some());
    assert!(sensor.configuration_xml.starts_with("<camera name="));
}

#[test]
fn native_link_wide_angle_camera_fragment_values_are_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let xml = fragment(fixture, "wide_angle_camera_sensor", "wideanglecamera");
    let sdf = load_str(&xml, "sensors.sdf#wide-angle", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "wide_angle_camera_sensor");
    assert_eq!(sensor.kind, "wideanglecamera");
    assert_eq!(sensor.effective_type(), Some(SensorType::WideAngleCamera));
    assert!(matches!(sensor.config, SensorConfig::Camera(_)));
    assert_eq!(sensor.pose.transform.translation, [20., 30., 40.]);
    assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(!sensor.enable_metrics);
    let camera = sensor.camera().unwrap();
    assert_eq!(camera.name, "wide_angle_cam");
    assert_eq!(camera.pose.transform.translation, [0.2, 0.3, 0.4]);
    assert_eq!(camera.pose.transform.rotation, [0., 0., 0., 1.]);
    assert_eq!(camera.horizontal_fov, "3.14".parse::<f64>().unwrap());
    assert_eq!(sensor.topic, "wideanglecamera");
    assert_eq!(camera.trigger_topic, "");
    assert!(!camera.triggered);
    assert_eq!(camera.image_width, 320);
    assert_eq!(camera.image_height, 240);
    assert_eq!(camera.pixel_format, CameraPixelFormat::RgbInt8);
    assert_eq!(camera.near_clip, 0.1);
    assert_eq!(camera.far_clip, 100.);
    assert_eq!(camera.lens.kind, "custom");
    assert!(camera.lens.scale_to_hfov);
    assert_eq!(camera.lens.c1, 1.05);
    assert_eq!(camera.lens.c2, 4.);
    assert_eq!(camera.lens.c3, 0.);
    assert_eq!(camera.lens.focal_length, 1.);
    assert_eq!(camera.lens.function, "tan");
    assert_eq!(camera.lens.cutoff_angle, "3.1415".parse::<f64>().unwrap());
    assert_eq!(camera.lens.environment_texture_size, 512);
    assert!(sensor.always_on);
    assert_eq!(sensor.update_rate, 30.);
    assert!(camera.source.is_some());
    assert!(sensor.configuration_xml.starts_with("<camera name="));
}

#[test]
fn native_thermal_and_wide_angle_aliases_keep_rust_mismatch_policy() {
    for (kind, effective) in [
        ("thermal", SensorType::ThermalCamera),
        ("thermal_camera", SensorType::ThermalCamera),
        ("wideanglecamera", SensorType::WideAngleCamera),
        ("wide_angle_camera", SensorType::WideAngleCamera),
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
