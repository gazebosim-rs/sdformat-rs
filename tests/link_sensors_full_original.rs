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
// Pinned test/integration/link_dom.cc DOMLink.Sensors, unchanged 27-sensor fixture.
use sdformat::{CameraPixelFormat, NoResources, Noise, Sensor, SensorType, Transform, load_str};

fn pose(actual: Transform, expected: [f64; 3]) {
    for (actual, expected) in actual.translation.into_iter().zip(expected) {
        assert!((actual - expected).abs() <= 1e-3);
    }
    for (actual, expected) in actual.rotation.into_iter().zip([0., 0., 0., 1.]) {
        assert!((actual - expected).abs() <= 1e-3);
    }
}

fn by_name<'a>(sensors: &'a [Sensor], name: &str) -> &'a Sensor {
    sensors.iter().find(|sensor| sensor.name == name).unwrap()
}

fn noise(actual: &Noise, mean: f64, stddev: f64) {
    assert_eq!(actual.mean, mean);
    assert_eq!(actual.stddev, stddev);
}

#[test]
fn original_dom_link_sensors_all_assertions() {
    let sdf = load_str(
        include_str!("fixtures/sensors.sdf"),
        "sensors.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().unwrap();
    assert_eq!(model.name, "model");
    let link = model.links.first().unwrap();
    assert_eq!(link.name, "link");
    let sensors = &link.sensors;
    assert_eq!(sensors.len(), 27);
    assert_eq!(sensors.first().unwrap().name, "altimeter_sensor");
    assert!(sensors.iter().any(|sensor| sensor.name == "camera_sensor"));
    assert!(
        !sensors
            .iter()
            .any(|sensor| sensor.name == "bad_camera_sensor")
    );
    assert!(
        sensors
            .iter()
            .any(|sensor| sensor.name == "wide_angle_camera_sensor")
    );

    // Every native ByName lookup, effective type, raw pose and metrics assertion.
    // `None` denotes an original with no assertion on that particular field.
    for (name, kind, effective, raw_pose, metrics) in [
        (
            "altimeter_sensor",
            "altimeter",
            SensorType::Altimeter,
            Some([0., 0., 0.]),
            Some(false),
        ),
        (
            "camera_sensor",
            "camera",
            SensorType::Camera,
            Some([1., 2., 3.]),
            Some(false),
        ),
        (
            "contact_sensor",
            "contact",
            SensorType::Contact,
            Some([4., 5., 6.]),
            Some(true),
        ),
        ("custom_sensor", "custom", SensorType::Custom, None, None),
        (
            "depth_sensor",
            "depth",
            SensorType::DepthCamera,
            Some([7., 8., 9.]),
            Some(true),
        ),
        (
            "force_torque_sensor",
            "force_torque",
            SensorType::ForceTorque,
            Some([10., 11., 12.]),
            Some(false),
        ),
        (
            "navsat_sensor",
            "gps",
            SensorType::NavSat,
            Some([13., 14., 15.]),
            Some(false),
        ),
        (
            "gpu_ray_sensor",
            "gpu_ray",
            SensorType::GpuLidar,
            Some([1., 2., 3.]),
            Some(false),
        ),
        (
            "gpu_lidar_sensor",
            "gpu_lidar",
            SensorType::GpuLidar,
            Some([1., 2., 3.]),
            Some(false),
        ),
        (
            "imu_sensor",
            "imu",
            SensorType::Imu,
            Some([4., 5., 6.]),
            Some(false),
        ),
        (
            "logical_camera_sensor",
            "logical_camera",
            SensorType::LogicalCamera,
            Some([7., 8., 9.]),
            Some(false),
        ),
        (
            "magnetometer_sensor",
            "magnetometer",
            SensorType::Magnetometer,
            Some([10., 11., 12.]),
            Some(false),
        ),
        (
            "multicamera_sensor",
            "multicamera",
            SensorType::Multicamera,
            Some([13., 14., 15.]),
            Some(false),
        ),
        (
            "ray_sensor",
            "ray",
            SensorType::Lidar,
            Some([1., 2., 3.]),
            Some(false),
        ),
        (
            "lidar_sensor",
            "lidar",
            SensorType::Lidar,
            Some([1., 2., 3.]),
            Some(true),
        ),
        (
            "rfid_sensor",
            "rfid",
            SensorType::Rfid,
            Some([4., 5., 6.]),
            Some(false),
        ),
        (
            "rfid_tag",
            "rfidtag",
            SensorType::RfidTag,
            Some([7., 8., 9.]),
            Some(false),
        ),
        (
            "rgbd_sensor",
            "rgbd",
            SensorType::RgbdCamera,
            Some([37., 38., 39.]),
            Some(false),
        ),
        (
            "sonar_sensor",
            "sonar",
            SensorType::Sonar,
            Some([10., 11., 12.]),
            Some(false),
        ),
        (
            "thermal_sensor",
            "thermal",
            SensorType::ThermalCamera,
            Some([37., 38., 39.]),
            Some(false),
        ),
        (
            "segmentation_sensor",
            "segmentation",
            SensorType::SegmentationCamera,
            Some([37., 38., 39.]),
            None,
        ),
        (
            "boundingbox_sensor",
            "boundingbox",
            SensorType::BoundingBoxCamera,
            Some([37., 38., 39.]),
            None,
        ),
        (
            "wireless_receiver",
            "wireless_receiver",
            SensorType::WirelessReceiver,
            Some([13., 14., 15.]),
            Some(false),
        ),
        (
            "wireless_transmitter",
            "wireless_transmitter",
            SensorType::WirelessTransmitter,
            Some([1., 2., 3.]),
            Some(false),
        ),
        (
            "air_pressure_sensor",
            "air_pressure",
            SensorType::AirPressure,
            Some([10., 20., 30.]),
            Some(false),
        ),
        (
            "air_speed_sensor",
            "air_speed",
            SensorType::AirSpeed,
            Some([11., 22., 33.]),
            Some(false),
        ),
        (
            "wide_angle_camera_sensor",
            "wideanglecamera",
            SensorType::WideAngleCamera,
            Some([20., 30., 40.]),
            Some(false),
        ),
    ] {
        let sensor = by_name(sensors, name);
        assert_eq!(sensor.name, name);
        assert_eq!(sensor.kind, kind, "{name}");
        assert_eq!(sensor.effective_type(), Some(effective), "{name}");
        if let Some(raw_pose) = raw_pose {
            pose(sensor.pose.transform, raw_pose);
        }
        if let Some(metrics) = metrics {
            assert_eq!(sensor.enable_metrics, metrics, "{name}");
        }
    }

    let altimeter = by_name(sensors, "altimeter_sensor").altimeter().unwrap();
    noise(&altimeter.vertical_position, 0.1, 0.2);
    noise(&altimeter.vertical_velocity, 2.3, 4.5);

    let camera_sensor = by_name(sensors, "camera_sensor");
    let camera = camera_sensor.camera().unwrap();
    assert_eq!(camera.name, "my_camera");
    pose(camera.pose.transform, [0.1, 0.2, 0.3]);
    assert_eq!(camera.horizontal_fov, 0.75);
    assert_eq!(camera_sensor.topic, "");
    assert_eq!(camera.trigger_topic, "my_camera/trigger");
    assert!(camera.triggered);
    assert_eq!(camera.camera_info_topic, "/camera_sensor/camera_info");
    assert_eq!((camera.image_width, camera.image_height), (640, 480));
    assert_eq!(camera.pixel_format, CameraPixelFormat::RgbInt8);
    assert_eq!((camera.near_clip, camera.far_clip), (0.2, 12.3));
    assert!(camera.save_frames);
    assert_eq!(camera.save_frames_path, "/tmp/cam");
    noise(&camera.image_noise, 0.5, 0.1);
    assert_eq!(
        (
            camera.distortion.k1,
            camera.distortion.k2,
            camera.distortion.k3
        ),
        (0.1, 0.2, 0.3)
    );
    assert_eq!((camera.distortion.p1, camera.distortion.p2), (0.4, 0.5));
    assert_eq!(camera.distortion.center, [0.2, 0.4]);
    assert_eq!(camera.lens.kind, "custom");
    assert!(!camera.lens.scale_to_hfov);
    assert_eq!(
        (camera.lens.c1, camera.lens.c2, camera.lens.c3),
        (1.1, 2.2, 3.3)
    );
    assert_eq!(camera.lens.focal_length, 1.2);
    assert_eq!(camera.lens.function, "sin");
    assert_eq!(camera.lens.cutoff_angle, 0.7505);
    assert_eq!(camera.lens.environment_texture_size, 128);
    assert_eq!(
        (
            camera.lens.intrinsics.fx,
            camera.lens.intrinsics.fy,
            camera.lens.intrinsics.cx,
            camera.lens.intrinsics.cy,
            camera.lens.intrinsics.skew
        ),
        (280., 281., 162., 124., 1.2)
    );
    assert_eq!(
        (
            camera.lens.projection.fx,
            camera.lens.projection.fy,
            camera.lens.projection.cx,
            camera.lens.projection.cy,
            camera.lens.projection.tx,
            camera.lens.projection.ty
        ),
        (280., 281., 162., 124., 0., 0.)
    );

    let contact = by_name(sensors, "contact_sensor");
    pose(
        sdf.resolve_pose(
            &contact.pose,
            &model.scoped_name,
            &link.scoped_name,
            &model.scoped_name,
        )
        .unwrap(),
        [4., 5., 6.],
    );
    pose(
        sdf.resolve_pose(
            &contact.pose,
            &model.scoped_name,
            &link.scoped_name,
            &link.scoped_name,
        )
        .unwrap(),
        [4., 5., 3.],
    );

    let custom = by_name(sensors, "custom_sensor");
    assert!(!custom.configuration_xml.is_empty());
    let extension = custom.custom().unwrap();
    assert_eq!(extension.element_name, "my_namespace:custom_sensor");
    assert_eq!(
        extension
            .parameter_text("some_param")
            .unwrap()
            .parse::<i32>()
            .unwrap(),
        123
    );

    for (name, camera_name) in [
        ("depth_sensor", "my_depth_camera"),
        ("rgbd_sensor", "my_rgbd_camera"),
        ("thermal_sensor", "my_thermal_camera"),
        ("segmentation_sensor", "my_segmentation_camera"),
        ("boundingbox_sensor", "my_boundingbox_camera"),
    ] {
        assert_eq!(by_name(sensors, name).camera().unwrap().name, camera_name);
    }
    let rgbd = by_name(sensors, "rgbd_sensor");
    assert_eq!(rgbd.topic, "");
    assert_eq!(rgbd.camera().unwrap().trigger_topic, "");
    assert!(rgbd.camera().unwrap().triggered);
    let segmentation = by_name(sensors, "segmentation_sensor").camera().unwrap();
    assert!(segmentation.has_segmentation_type);
    assert_eq!(segmentation.segmentation_type, "semantic");
    let boundingbox = by_name(sensors, "boundingbox_sensor").camera().unwrap();
    assert!(boundingbox.has_bounding_box_type);
    assert_eq!(boundingbox.bounding_box_type, "2d");

    let navsat = by_name(sensors, "navsat_sensor").navsat().unwrap();
    noise(&navsat.horizontal_position, 1.2, 3.4);
    noise(&navsat.vertical_position, 5.6, 7.8);
    noise(&navsat.horizontal_velocity, 9.1, 10.11);
    noise(&navsat.vertical_velocity, 12.13, 14.15);
    assert!(by_name(sensors, "gpu_ray_sensor").lidar().is_some());
    assert!(by_name(sensors, "gpu_lidar_sensor").lidar().is_some());

    let imu = by_name(sensors, "imu_sensor").imu().unwrap();
    for (actual, expected) in imu
        .linear_acceleration
        .iter()
        .chain(&imu.angular_velocity)
        .zip([
            [0., 0.1, 0.2, 1.],
            [1., 1.1, 1.2, 2.],
            [2., 2.1, 2.2, 3.],
            [3., 3.1, 4.2, 4.],
            [4., 4.1, 5.2, 5.],
            [5., 5.1, 6.2, 6.],
        ])
    {
        assert_eq!(
            [
                actual.mean,
                actual.stddev,
                actual.dynamic_bias_stddev,
                actual.dynamic_bias_correlation_time
            ],
            expected
        );
    }
    assert_eq!(imu.localization, "ENU");
    assert_eq!(imu.custom_rpy_parent_frame, "linka");
    assert_eq!(imu.custom_rpy, [0., 1., 0.]);
    assert_eq!(imu.gravity_dir_x_parent_frame, "linkb");
    assert_eq!(imu.gravity_dir_x, [0., 0., 1.]);
    assert!(!imu.enable_orientation);

    let magnetometer = by_name(sensors, "magnetometer_sensor")
        .magnetometer()
        .unwrap();
    for (actual, expected) in magnetometer
        .noise
        .iter()
        .zip([(0.1, 0.2), (1.2, 2.3), (3.4, 5.6)])
    {
        noise(actual, expected.0, expected.1);
    }
    for name in ["ray_sensor", "lidar_sensor"] {
        let lidar = by_name(sensors, name).lidar().unwrap();
        assert_eq!(
            (
                lidar.horizontal.samples,
                lidar.horizontal.resolution,
                lidar.horizontal.min_angle,
                lidar.horizontal.max_angle
            ),
            (320, 0.9, 1.75, 2.94)
        );
        assert_eq!(
            (
                lidar.vertical.samples,
                lidar.vertical.resolution,
                lidar.vertical.min_angle,
                lidar.vertical.max_angle
            ),
            (240, 0.8, 2.75, 3.94)
        );
        assert_eq!(
            (lidar.range_min, lidar.range_max, lidar.range_resolution),
            (1.23, 4.56, 7.89)
        );
        noise(&lidar.noise, 0.98, 0.76);
    }

    let pressure = by_name(sensors, "air_pressure_sensor")
        .air_pressure()
        .unwrap();
    noise(&pressure.pressure, 3.4, 5.6);
    assert_eq!(pressure.reference_altitude, 123.4);
    noise(
        &by_name(sensors, "air_speed_sensor")
            .air_speed()
            .unwrap()
            .pressure,
        0.,
        0.01,
    );

    let wide_sensor = by_name(sensors, "wide_angle_camera_sensor");
    let wide = wide_sensor.camera().unwrap();
    assert_eq!(wide.name, "wide_angle_cam");
    pose(wide.pose.transform, [0.2, 0.3, 0.4]);
    assert_eq!(wide.horizontal_fov, "3.14".parse::<f64>().unwrap());
    assert_eq!(wide_sensor.topic, "wideanglecamera");
    assert_eq!(wide.trigger_topic, "");
    assert!(!wide.triggered);
    assert_eq!((wide.image_width, wide.image_height), (320, 240));
    assert_eq!(wide.pixel_format, CameraPixelFormat::RgbInt8);
    assert_eq!((wide.near_clip, wide.far_clip), (0.1, 100.));
    assert_eq!(wide.lens.kind, "custom");
    assert!(wide.lens.scale_to_hfov);
    assert_eq!((wide.lens.c1, wide.lens.c2, wide.lens.c3), (1.05, 4., 0.));
    assert_eq!(wide.lens.focal_length, 1.);
    assert_eq!(wide.lens.function, "tan");
    assert_eq!(wide.lens.cutoff_angle, "3.1415".parse::<f64>().unwrap());
    assert_eq!(wide.lens.environment_texture_size, 512);
}
