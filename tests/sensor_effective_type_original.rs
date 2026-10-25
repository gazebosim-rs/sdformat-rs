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

use sdformat::SensorType;

#[test]
fn native_sensor_type_strings_have_the_original_value_mapping() {
    // DOMSensor.Type tests native SetType(Type), Type and TypeStr for these
    // values. The Rust enum retains their value/string observations; mutable
    // Sensor::SetType state, including invalid-string preservation, is pending.
    let original = [
        (SensorType::None, "none"),
        (SensorType::Altimeter, "altimeter"),
        (SensorType::BoundingBoxCamera, "boundingbox_camera"),
        (SensorType::Camera, "camera"),
        (SensorType::Contact, "contact"),
        (SensorType::DepthCamera, "depth_camera"),
        (SensorType::ForceTorque, "force_torque"),
        (SensorType::NavSat, "navsat"),
        (SensorType::GpuLidar, "gpu_lidar"),
        (SensorType::Imu, "imu"),
        (SensorType::LogicalCamera, "logical_camera"),
        (SensorType::Magnetometer, "magnetometer"),
        (SensorType::Multicamera, "multicamera"),
        (SensorType::Lidar, "lidar"),
        (SensorType::Rfid, "rfid"),
        (SensorType::RfidTag, "rfidtag"),
        (SensorType::SegmentationCamera, "segmentation_camera"),
        (SensorType::Sonar, "sonar"),
        (SensorType::WirelessReceiver, "wireless_receiver"),
        (SensorType::WirelessTransmitter, "wireless_transmitter"),
        (SensorType::ThermalCamera, "thermal_camera"),
        (SensorType::Custom, "custom"),
        (SensorType::WideAngleCamera, "wide_angle_camera"),
    ];
    assert_eq!(SensorType::default(), SensorType::None);
    for (kind, name) in original {
        assert_eq!(kind.as_str(), name);
        assert_eq!(SensorType::from_type_str(name), Some(kind));
    }
    assert_eq!(SensorType::from_type_str("bad_sensor_type"), None);
}

#[test]
fn native_sensor_type_extensions_and_xml_aliases_are_explicit() {
    // Native's 27-entry string table has four entries absent from the older
    // DOMSensor.Type original vector. These are supplemental checks.
    for (kind, name) in [
        (SensorType::Gps, "gps"),
        (SensorType::AirPressure, "air_pressure"),
        (SensorType::RgbdCamera, "rgbd_camera"),
        (SensorType::AirSpeed, "air_speed"),
    ] {
        assert_eq!(kind.as_str(), name);
        assert_eq!(SensorType::from_type_str(name), Some(kind));
    }
    for (xml, effective) in [
        ("gps", SensorType::NavSat),
        ("navsat", SensorType::NavSat),
        ("gpu_ray", SensorType::GpuLidar),
        ("gpu_lidar", SensorType::GpuLidar),
        ("ray", SensorType::Lidar),
        ("lidar", SensorType::Lidar),
        ("depth", SensorType::DepthCamera),
        ("depth_camera", SensorType::DepthCamera),
        ("rgbd", SensorType::RgbdCamera),
        ("rgbd_camera", SensorType::RgbdCamera),
        ("thermal", SensorType::ThermalCamera),
        ("thermal_camera", SensorType::ThermalCamera),
        ("segmentation", SensorType::SegmentationCamera),
        ("segmentation_camera", SensorType::SegmentationCamera),
        ("boundingbox", SensorType::BoundingBoxCamera),
        ("boundingbox_camera", SensorType::BoundingBoxCamera),
        ("wideanglecamera", SensorType::WideAngleCamera),
        ("wide_angle_camera", SensorType::WideAngleCamera),
        ("camera", SensorType::Camera),
        ("magnetometer", SensorType::Magnetometer),
    ] {
        assert_eq!(SensorType::from_xml_kind(xml), Some(effective), "{xml}");
    }
    assert_eq!(SensorType::from_xml_kind("bad_sensor_type"), None);
    assert_eq!(SensorType::from_xml_kind("none"), None);
}
