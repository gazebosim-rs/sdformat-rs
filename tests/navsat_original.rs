/*
 * Copyright (C) 2021 Open Source Robotics Foundation
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

use sdformat::{ErrorCode, NavSat, NoResources, Noise, SensorConfig, SensorType, load_str};

#[test]
fn original_navsat_construction() {
    let nav = NavSat::default();
    let default_noise = Noise::default();
    assert_eq!(nav.vertical_position, default_noise);
    assert_eq!(nav.horizontal_position, default_noise);
    assert_eq!(nav.vertical_velocity, default_noise);
    assert_eq!(nav.horizontal_velocity, default_noise);
}

#[test]
fn original_navsat_set_copy_and_move() {
    let mut nav = NavSat::default();
    let default_noise = Noise::default();
    let noise = Noise {
        mean: 6.5,
        stddev: 3.79,
        ..Noise::default()
    };
    nav.vertical_position = noise;
    assert_eq!(nav.vertical_position, noise);
    assert_eq!(nav.horizontal_position, default_noise);
    assert_eq!(nav.vertical_velocity, default_noise);
    assert_eq!(nav.horizontal_velocity, default_noise);
    nav.horizontal_position = noise;
    assert_eq!(nav.vertical_position, noise);
    assert_eq!(nav.horizontal_position, noise);
    assert_eq!(nav.vertical_velocity, default_noise);
    assert_eq!(nav.horizontal_velocity, default_noise);
    nav.vertical_velocity = noise;
    assert_eq!(nav.vertical_position, noise);
    assert_eq!(nav.horizontal_position, noise);
    assert_eq!(nav.vertical_velocity, noise);
    assert_eq!(nav.horizontal_velocity, default_noise);
    nav.horizontal_velocity = noise;
    assert_eq!(nav.vertical_position, noise);
    assert_eq!(nav.horizontal_position, noise);
    assert_eq!(nav.vertical_velocity, noise);
    assert_eq!(nav.horizontal_velocity, noise);

    let nav2 = NavSat::default();
    assert_ne!(nav2, nav);
    let mut nav_copied = nav.clone();
    assert_eq!(nav_copied, nav);
    let mut nav_assigned = NavSat::default();
    nav_assigned.clone_from(&nav);
    assert_eq!(nav_assigned, nav);
    let nav_moved = nav;
    assert_eq!(nav_copied, nav_moved);
    nav = nav_moved.clone();
    assert_eq!(nav_copied, nav);
    let mut nav_move_assigned = NavSat::default();
    assert_eq!(nav_move_assigned, nav2);
    nav_move_assigned = nav_copied;
    assert_eq!(nav_assigned, nav_move_assigned);
    nav_copied = nav_move_assigned.clone();
    assert_eq!(nav_assigned, nav_copied);
}

fn sensor_xml(child: &str, kind: &str) -> String {
    format!(
        "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='{kind}'>{child}</sensor></link></model></sdf>"
    )
}

#[test]
fn native_navsat_gps_sensor_fragment_is_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let start = fixture
        .find("<sensor name=\"navsat_sensor\" type=\"gps\">")
        .unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let exact_sensor = &fixture[start..end];
    let xml = format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{exact_sensor}</link></model></sdf>"
    );
    let sdf = load_str(&xml, "sensors.sdf#navsat_sensor", &NoResources).unwrap();
    let sensor = &sdf.models[0].links[0].sensors[0];
    assert_eq!(sensor.name, "navsat_sensor");
    assert_eq!(sensor.kind, "gps", "literal legacy type remains owned");
    assert_eq!(sensor.effective_type(), Some(SensorType::NavSat));
    assert!(matches!(sensor.config, SensorConfig::NavSat(_)));
    assert_eq!(sensor.pose.transform.translation, [13., 14., 15.]);
    assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(!sensor.enable_metrics);
    let nav = sensor.navsat().unwrap();
    assert!(nav.source.is_some());
    assert_eq!(nav.horizontal_position.mean, 1.2);
    assert_eq!(nav.horizontal_position.stddev, 3.4);
    assert_eq!(nav.vertical_position.mean, 5.6);
    assert_eq!(nav.vertical_position.stddev, 7.8);
    assert_eq!(nav.horizontal_velocity.mean, 9.1);
    assert_eq!(nav.horizontal_velocity.stddev, 10.11);
    assert_eq!(nav.vertical_velocity.mean, 12.13);
    assert_eq!(nav.vertical_velocity.stddev, 14.15);
    assert!(sensor.configuration_xml.starts_with("<gps>"));

    let default = load_str(
        &sensor_xml("", "navsat"),
        "default-navsat.sdf",
        &NoResources,
    )
    .unwrap();
    let sensor = &default.models[0].links[0].sensors[0];
    assert_eq!(sensor.navsat().unwrap(), &NavSat::default());
    assert!(sensor.configuration_xml.is_empty());
    let explicit = load_str(
        &sensor_xml(
            "<navsat><position_sensing><horizontal/></position_sensing></navsat>",
            "navsat",
        ),
        "empty-navsat.sdf",
        &NoResources,
    )
    .unwrap();
    let nav = explicit.models[0].links[0].sensors[0].navsat().unwrap();
    assert_eq!(nav.horizontal_position, Noise::default());
    assert!(nav.source.is_some());
    let dual = load_str(
        &sensor_xml(
            "<gps/><navsat><position_sensing><horizontal><noise type='gaussian'><mean>2</mean></noise></horizontal></position_sensing></navsat>",
            "gps",
        ),
        "dual-navsat.sdf",
        &NoResources,
    )
    .unwrap();
    let sensor = &dual.models[0].links[0].sensors[0];
    assert_eq!(sensor.kind, "gps");
    assert_eq!(sensor.navsat().unwrap().horizontal_position.mean, 2.);
    assert!(sensor.configuration_xml.starts_with("<navsat>"));
    let imu = load_str(&sensor_xml("", "imu"), "default-imu.sdf", &NoResources).unwrap();
    assert!(imu.models[0].links[0].sensors[0].navsat().is_none());
    for (kind, child) in [("imu", "gps"), ("gps", "imu")] {
        let xml = sensor_xml(&format!("<{child}/>"), kind);
        assert_eq!(
            load_str(&xml, "mismatched-navsat.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}
