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

use sdformat::{ErrorCode, Lidar, NoResources, NoiseKind, SensorConfig, SensorType, load_str};

fn sensor_xml(child: &str, kind: &str) -> String {
    format!(
        "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'>{child}</sensor></link></model></sdf>"
    )
}

fn fixture_sensor(fixture: &str, name: &str, kind: &str) -> String {
    let tag = format!("<sensor name=\"{name}\" type=\"{kind}\">");
    let start = fixture.find(&tag).unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{}</link></model></sdf>",
        &fixture[start..end]
    )
}

#[test]
fn native_lidar_link_fragments_are_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    for (name, kind, metrics) in [
        ("gpu_ray_sensor", "gpu_ray", false),
        ("gpu_lidar_sensor", "gpu_lidar", false),
        ("ray_sensor", "ray", false),
        ("lidar_sensor", "lidar", true),
    ] {
        let xml = fixture_sensor(fixture, name, kind);
        let sdf = load_str(&xml, "sensors.sdf#lidar", &NoResources).unwrap();
        let sensor = &sdf.models[0].links[0].sensors[0];
        assert_eq!(sensor.name, name);
        assert_eq!(sensor.kind, kind);
        assert_eq!(
            sensor.effective_type(),
            Some(if kind.starts_with("gpu_") {
                SensorType::GpuLidar
            } else {
                SensorType::Lidar
            })
        );
        assert!(matches!(sensor.config, SensorConfig::Lidar(_)));
        assert_eq!(sensor.pose.transform.translation, [1., 2., 3.]);
        assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
        assert_eq!(sensor.enable_metrics, metrics);
        let lidar = sensor.lidar().unwrap();
        if name == "gpu_ray_sensor" {
            assert_eq!(lidar, &Lidar::default());
            assert!(sensor.configuration_xml.is_empty());
        } else {
            assert!(lidar.source.is_some());
            if name == "gpu_lidar_sensor" {
                assert_eq!(lidar.horizontal.samples, 640);
                assert_eq!(lidar.vertical.samples, 1);
                assert_eq!(lidar.range_min, 0.08);
                assert_eq!(lidar.range_max, 10.);
            } else {
                assert_eq!(lidar.horizontal.samples, 320);
                assert_eq!(lidar.horizontal.resolution, 0.9);
                assert_eq!(lidar.horizontal.min_angle, 1.75);
                assert_eq!(lidar.horizontal.max_angle, 2.94);
                assert_eq!(lidar.vertical.samples, 240);
                assert_eq!(lidar.vertical.resolution, 0.8);
                assert_eq!(lidar.vertical.min_angle, 2.75);
                assert_eq!(lidar.vertical.max_angle, 3.94);
                assert_eq!(lidar.range_min, 1.23);
                assert_eq!(lidar.range_max, 4.56);
                assert_eq!(lidar.range_resolution, 7.89);
                assert_eq!(lidar.noise.kind, NoiseKind::Gaussian);
                assert_eq!(lidar.noise.mean, 0.98);
                assert_eq!(lidar.noise.stddev, 0.76);
            }
            assert!(sensor.configuration_xml.starts_with(if kind == "lidar" {
                "<lidar>"
            } else {
                "<ray>"
            }));
        }
    }
}

#[test]
fn lidar_parser_defaults_and_errors_are_explicit() {
    let empty = load_str(&sensor_xml("", "lidar"), "empty-lidar.sdf", &NoResources).unwrap();
    assert_eq!(
        empty.models[0].links[0].sensors[0].lidar().unwrap(),
        &Lidar::default()
    );
    let dual = load_str(
        &sensor_xml(
            "<ray/><lidar><scan><horizontal><samples>3</samples></horizontal></scan><range/></lidar>",
            "gpu_ray",
        ),
        "dual-lidar.sdf",
        &NoResources,
    )
    .unwrap();
    let sensor = &dual.models[0].links[0].sensors[0];
    assert_eq!(sensor.kind, "gpu_ray");
    assert_eq!(sensor.lidar().unwrap().horizontal.samples, 3);
    assert!(sensor.configuration_xml.starts_with("<lidar>"));
    let imu = load_str(&sensor_xml("", "imu"), "empty-imu.sdf", &NoResources).unwrap();
    assert!(imu.models[0].links[0].sensors[0].lidar().is_none());
    for (kind, child) in [("imu", "lidar"), ("lidar", "imu")] {
        let xml = sensor_xml(&format!("<{child}/>"), kind);
        assert_eq!(
            load_str(&xml, "mismatched-lidar.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let no_scan = sensor_xml("<lidar><range/></lidar>", "lidar");
    assert_eq!(
        load_str(&no_scan, "no-scan.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Missing
    );
    let no_horizontal = sensor_xml("<lidar><scan/><range/></lidar>", "lidar");
    assert_eq!(
        load_str(&no_horizontal, "no-horizontal.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Missing
    );
    let no_range = sensor_xml("<lidar><scan><horizontal/></scan></lidar>", "lidar");
    assert_eq!(
        load_str(&no_range, "no-range.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Missing
    );
    let invalid_scan = sensor_xml(
        "<lidar><scan><horizontal><unknown/></horizontal></scan><range/></lidar>",
        "lidar",
    );
    assert_eq!(
        load_str(&invalid_scan, "invalid-scan.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let bad_noise = sensor_xml(
        "<lidar><scan><horizontal/></scan><range/><noise><type>bad</type></noise></lidar>",
        "lidar",
    );
    assert_eq!(
        load_str(&bad_noise, "bad-noise.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
    let unknown_noise = sensor_xml(
        "<lidar><scan><horizontal/></scan><range/><noise><unknown/></noise></lidar>",
        "lidar",
    );
    assert_eq!(
        load_str(&unknown_noise, "unknown-noise.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let negative_noise = sensor_xml(
        "<lidar><scan><horizontal/></scan><range/><noise><type>gaussian</type><stddev>-1</stddev></noise></lidar>",
        "lidar",
    );
    assert_eq!(
        load_str(&negative_noise, "negative-noise.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
}
