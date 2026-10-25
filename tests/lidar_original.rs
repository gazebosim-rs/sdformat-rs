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

use sdformat::{Lidar, Noise};

#[test]
fn original_lidar_construction() {
    let lidar = Lidar::default();
    assert_eq!(lidar.noise, Noise::default());
}

#[test]
fn original_lidar_set_copy_and_move() {
    let mut lidar = Lidar::default();
    lidar.horizontal.samples = 123;
    assert_eq!(lidar.horizontal.samples, 123);
    lidar.horizontal.resolution = 0.45;
    assert_eq!(lidar.horizontal.resolution, 0.45);
    lidar.horizontal.min_angle = 0.67;
    assert_eq!(lidar.horizontal.min_angle, 0.67);
    lidar.horizontal.max_angle = 0.89;
    assert_eq!(lidar.horizontal.max_angle, 0.89);
    lidar.vertical.samples = 98;
    assert_eq!(lidar.vertical.samples, 98);
    lidar.vertical.resolution = 0.76;
    assert_eq!(lidar.vertical.resolution, 0.76);
    lidar.vertical.min_angle = 0.54;
    assert_eq!(lidar.vertical.min_angle, 0.54);
    lidar.vertical.max_angle = 0.321;
    assert_eq!(lidar.vertical.max_angle, 0.321);
    lidar.range_min = 1.2;
    assert_eq!(lidar.range_min, 1.2);
    lidar.range_max = 3.4;
    assert_eq!(lidar.range_max, 3.4);
    lidar.range_resolution = 5.6;
    assert_eq!(lidar.range_resolution, 5.6);
    let noise = Noise {
        mean: 6.5,
        stddev: 3.79,
        ..Noise::default()
    };
    lidar.noise = noise;
    assert_eq!(lidar.noise, noise);
    lidar.horizontal.samples = 111;
    lidar.horizontal.resolution = 2.2;
    assert_eq!(lidar.visibility_mask, u32::MAX);
    lidar.visibility_mask = 123;
    assert_eq!(lidar.visibility_mask, 123);

    let default = Lidar::default();
    assert_ne!(default, lidar);
    let mut copied = lidar.clone();
    assert_eq!(copied, lidar);
    let mut assigned = Lidar::default();
    assigned.clone_from(&lidar);
    assert_eq!(assigned, lidar);
    let moved = lidar;
    assert_eq!(copied, moved);
    lidar = moved.clone();
    assert_eq!(copied, lidar);
    let mut move_assigned = Lidar::default();
    assert_eq!(move_assigned, default);
    move_assigned = copied;
    assert_eq!(assigned, move_assigned);
    copied = move_assigned.clone();
    assert_eq!(assigned, copied);
}
