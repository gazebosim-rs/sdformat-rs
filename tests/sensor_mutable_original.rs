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
// Pinned src/Sensor_TEST.cc DOMSensor.MutableSensors, eight payload blocks.
use sdformat::{NoResources, Noise, Sensor, SensorConfig, load_str};

fn sensor(kind: &str) -> Sensor {
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='{kind}'/></link></model></sdf>"
    );
    let mut parsed = load_str(&xml, "mutable-sensor.sdf", &NoResources).unwrap();
    parsed.models.remove(0).links.remove(0).sensors.remove(0)
}

#[test]
fn original_dom_sensor_mutable_payload_observations() {
    let mut altimeter = sensor("altimeter");
    altimeter.config = SensorConfig::Altimeter(Box::default());
    let before = altimeter.altimeter().unwrap().vertical_position.mean;
    if let SensorConfig::Altimeter(value) = &mut altimeter.config {
        assert_eq!(value.vertical_position.mean, before);
        value.vertical_position = Noise {
            mean: 2.,
            ..Noise::default()
        };
        assert_eq!(value.vertical_position.mean, 2.);
    } else {
        panic!("altimeter payload");
    }
    assert_eq!(altimeter.altimeter().unwrap().vertical_position.mean, 2.);

    let mut pressure = sensor("air_pressure");
    pressure.config = SensorConfig::AirPressure(Box::default());
    let before = pressure.air_pressure().unwrap().reference_altitude;
    if let SensorConfig::AirPressure(value) = &mut pressure.config {
        assert_eq!(value.reference_altitude, before);
        value.reference_altitude = 2.;
        assert_eq!(value.reference_altitude, 2.);
    } else {
        panic!("air pressure payload");
    }
    assert_eq!(pressure.air_pressure().unwrap().reference_altitude, 2.);

    let mut camera = sensor("camera");
    camera.config = SensorConfig::Camera(Box::default());
    let before = camera.camera().unwrap().near_clip;
    if let SensorConfig::Camera(value) = &mut camera.config {
        assert_eq!(value.near_clip, before);
        value.near_clip = 2.;
        assert_eq!(value.near_clip, 2.);
    } else {
        panic!("camera payload");
    }
    assert_eq!(camera.camera().unwrap().near_clip, 2.);

    let mut force_torque = sensor("force_torque");
    force_torque.config = SensorConfig::ForceTorque(Box::default());
    let before = force_torque.force_torque().unwrap().force[0].mean;
    if let SensorConfig::ForceTorque(value) = &mut force_torque.config {
        assert_eq!(value.force[0].mean, before);
        value.force[0] = Noise {
            mean: 2.,
            ..Noise::default()
        };
        assert_eq!(value.force[0].mean, 2.);
    } else {
        panic!("force torque payload");
    }
    assert_eq!(force_torque.force_torque().unwrap().force[0].mean, 2.);

    // The native original sets FORCE_TORQUE type before installing an IMU.
    // Its assertions observe the independent IMU payload, not the type tag.
    let mut imu = sensor("force_torque");
    imu.config = SensorConfig::Imu(Box::default());
    assert_eq!(imu.kind, "force_torque");
    let before = imu.imu().unwrap().linear_acceleration[0].mean;
    if let SensorConfig::Imu(value) = &mut imu.config {
        assert_eq!(value.linear_acceleration[0].mean, before);
        value.linear_acceleration[0] = Noise {
            mean: 2.,
            ..Noise::default()
        };
        assert_eq!(value.linear_acceleration[0].mean, 2.);
    } else {
        panic!("IMU payload");
    }
    assert_eq!(imu.imu().unwrap().linear_acceleration[0].mean, 2.);

    let mut lidar = sensor("lidar");
    lidar.config = SensorConfig::Lidar(Box::default());
    let before = lidar.lidar().unwrap().noise.mean;
    if let SensorConfig::Lidar(value) = &mut lidar.config {
        assert_eq!(value.noise.mean, before);
        value.noise = Noise {
            mean: 2.,
            ..Noise::default()
        };
        assert_eq!(value.noise.mean, 2.);
    } else {
        panic!("Lidar payload");
    }
    assert_eq!(lidar.lidar().unwrap().noise.mean, 2.);

    let mut magnetometer = sensor("magnetometer");
    magnetometer.config = SensorConfig::Magnetometer(Box::default());
    let before = magnetometer.magnetometer().unwrap().noise[0].mean;
    if let SensorConfig::Magnetometer(value) = &mut magnetometer.config {
        assert_eq!(value.noise[0].mean, before);
        value.noise[0] = Noise {
            mean: 2.,
            ..Noise::default()
        };
        assert_eq!(value.noise[0].mean, 2.);
    } else {
        panic!("magnetometer payload");
    }
    assert_eq!(magnetometer.magnetometer().unwrap().noise[0].mean, 2.);

    let mut navsat = sensor("navsat");
    navsat.config = SensorConfig::NavSat(Box::default());
    let before = navsat.navsat().unwrap().horizontal_position.mean;
    if let SensorConfig::NavSat(value) = &mut navsat.config {
        assert_eq!(value.horizontal_position.mean, before);
        value.horizontal_position = Noise {
            mean: 2.,
            ..Noise::default()
        };
        assert_eq!(value.horizontal_position.mean, 2.);
    } else {
        panic!("NavSat payload");
    }
    assert_eq!(navsat.navsat().unwrap().horizontal_position.mean, 2.);
}
