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
    Camera, CameraPixelFormat, ErrorCode, NoResources, NoiseKind, SensorConfig, load_str,
};

fn fragment(fixture: &str) -> String {
    let tag = "<sensor name=\"camera_sensor\" type=\"camera\">";
    let start = fixture.find(tag).unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{}</link></model></sdf>",
        &fixture[start..end]
    )
}

#[test]
fn native_link_camera_fragment_values_are_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let xml = fragment(fixture);
    let sdf = load_str(&xml, "sensors.sdf#camera", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "camera_sensor");
    assert_eq!(sensor.kind, "camera");
    assert!(matches!(sensor.config, SensorConfig::Camera(_)));
    assert_eq!(sensor.pose.transform.translation, [1., 2., 3.]);
    assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(!sensor.enable_metrics);
    let camera = sensor.camera().unwrap();
    assert_eq!(camera.name, "my_camera");
    assert_eq!(camera.pose.transform.translation, [0.1, 0.2, 0.3]);
    assert_eq!(camera.pose.transform.rotation, [0., 0., 0., 1.]);
    assert_eq!(camera.horizontal_fov, 0.75);
    assert_eq!(sensor.topic, "");
    assert_eq!(camera.trigger_topic, "my_camera/trigger");
    assert!(camera.triggered);
    assert_eq!(camera.camera_info_topic, "/camera_sensor/camera_info");
    assert_eq!(camera.image_width, 640);
    assert_eq!(camera.image_height, 480);
    assert_eq!(camera.pixel_format, CameraPixelFormat::RgbInt8);
    assert_eq!(camera.near_clip, 0.2);
    assert_eq!(camera.far_clip, 12.3);
    assert!(camera.save_frames);
    assert_eq!(camera.save_frames_path, "/tmp/cam");
    assert_eq!(camera.image_noise.mean, 0.5);
    assert_eq!(camera.image_noise.stddev, 0.1);
    assert_eq!(camera.distortion.k1, 0.1);
    assert_eq!(camera.distortion.k2, 0.2);
    assert_eq!(camera.distortion.k3, 0.3);
    assert_eq!(camera.distortion.p1, 0.4);
    assert_eq!(camera.distortion.p2, 0.5);
    assert_eq!(camera.distortion.center, [0.2, 0.4]);
    assert_eq!(camera.lens.kind, "custom");
    assert!(!camera.lens.scale_to_hfov);
    assert_eq!(camera.lens.c1, 1.1);
    assert_eq!(camera.lens.c2, 2.2);
    assert_eq!(camera.lens.c3, 3.3);
    assert_eq!(camera.lens.focal_length, 1.2);
    assert_eq!(camera.lens.function, "sin");
    assert_eq!(camera.lens.cutoff_angle, 0.7505);
    assert_eq!(camera.lens.environment_texture_size, 128);
    assert_eq!(camera.lens.intrinsics.fx, 280.);
    assert_eq!(camera.lens.intrinsics.fy, 281.);
    assert_eq!(camera.lens.intrinsics.cx, 162.);
    assert_eq!(camera.lens.intrinsics.cy, 124.);
    assert_eq!(camera.lens.intrinsics.skew, 1.2);
    assert_eq!(camera.lens.projection.fx, 280.);
    assert_eq!(camera.lens.projection.fy, 281.);
    assert_eq!(camera.lens.projection.cx, 162.);
    assert_eq!(camera.lens.projection.cy, 124.);
    assert_eq!(camera.lens.projection.tx, 0.);
    assert_eq!(camera.lens.projection.ty, 0.);
    assert_eq!(camera.image_noise.kind, NoiseKind::Gaussian);
    assert!(camera.source.is_some());
    assert!(
        sensor
            .configuration_xml
            .starts_with("<camera name=\"my_camera\">")
    );
}

#[test]
fn camera_parser_defaults_and_mismatch_controls() {
    let empty = load_str(
        "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='camera'/></link></model></sdf>",
        "empty-camera.sdf", &NoResources,
    ).unwrap();
    let sensor = &empty.models[0].links[0].sensors[0];
    assert_eq!(sensor.camera().unwrap(), &Camera::default());
    assert!(sensor.configuration_xml.is_empty());
    assert!(sensor.imu().is_none());
    for child in [
        "<imu/>",
        "<camera><extra/></camera>",
        "<camera><image/><clip/><lens><extra/></lens></camera>",
        "<camera><image/><clip/><lens><projection><extra/></projection></lens></camera>",
    ] {
        let xml = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='camera'>{child}</sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "bad-camera.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let luma = "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='camera'><camera><image><format>L8</format></image><clip/><save enabled='false'/></camera></sensor></link></model></sdf>";
    let parsed = load_str(luma, "luma-camera.sdf", &NoResources).unwrap();
    let camera = parsed.models[0].links[0].sensors[0].camera().unwrap();
    assert_eq!(camera.pixel_format, CameraPixelFormat::LInt8);
    assert!(!camera.save_frames);

    let optional = "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='camera'><camera><image/><clip/><distortion/><lens/></camera></sensor></link></model></sdf>";
    let parsed = load_str(optional, "optional-camera.sdf", &NoResources).unwrap();
    let camera = parsed.models[0].links[0].sensors[0].camera().unwrap();
    assert_eq!(camera.image_width, Camera::default().image_width);
    assert_eq!(camera.distortion.center, [0.5, 0.5]);
    assert_eq!(camera.lens.projection.tx, 0.);
    assert!(camera.source.is_some());

    for (child, code) in [
        (
            "<camera><image/><clip/><save enabled='oops'/></camera>",
            ErrorCode::InvalidValue,
        ),
        (
            "<camera><image/><clip/><noise><type>bad</type></noise></camera>",
            ErrorCode::InvalidValue,
        ),
        (
            "<camera><image/><clip/><lens><env_texture_size>bad</env_texture_size></lens></camera>",
            ErrorCode::InvalidValue,
        ),
        (
            "<camera><image><format>bad</format></image><clip/></camera>",
            ErrorCode::InvalidValue,
        ),
        ("<camera/>", ErrorCode::Missing),
        ("<camera><image/></camera>", ErrorCode::Missing),
    ] {
        let xml = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='camera'>{child}</sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "invalid-camera.sdf", &NoResources)
                .unwrap_err()
                .code,
            code
        );
    }
    for (kind, expected) in [
        ("none", NoiseKind::None),
        ("gaussian_quantized", NoiseKind::GaussianQuantized),
    ] {
        let xml = format!(
            "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='camera'><camera><image/><clip/><noise><type>{kind}</type></noise></camera></sensor></link></model></sdf>"
        );
        let parsed = load_str(&xml, "camera-noise.sdf", &NoResources).unwrap();
        assert_eq!(
            parsed.models[0].links[0].sensors[0]
                .camera()
                .unwrap()
                .image_noise
                .kind,
            expected
        );
    }
    let imu = "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='imu'><imu/></sensor></link></model></sdf>";
    let parsed = load_str(imu, "imu.sdf", &NoResources).unwrap();
    assert!(parsed.models[0].links[0].sensors[0].camera().is_none());
}
