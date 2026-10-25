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

use sdformat::{AirSpeed, Noise, NoiseKind};

#[test]
fn original_air_speed_construction() {
    let speed = AirSpeed::default();
    assert_eq!(speed.pressure, Noise::default());
}

#[test]
fn original_air_speed_set_copy_and_move() {
    let mut speed = AirSpeed::default();
    assert_eq!(speed.pressure, Noise::default());
    let noise = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };
    speed.pressure = noise;
    assert_eq!(speed.pressure, noise);

    let mut copied = speed.clone();
    assert_eq!(speed, copied, "copy construction");
    let mut assigned = AirSpeed::default();
    assigned.clone_from(&speed);
    assert_eq!(speed, assigned, "copy assignment");
    let moved = speed;
    assert_eq!(copied, moved, "move construction");
    speed = moved.clone();
    assert_eq!(copied, speed, "copy assignment after move");
    let mut move_assigned = AirSpeed::default();
    assert_ne!(move_assigned, assigned);
    move_assigned = copied;
    assert_eq!(assigned, move_assigned, "move assignment");
    copied = move_assigned.clone();
    assert_eq!(assigned, copied, "copy assignment after move assignment");
    let mut default = AirSpeed::default();
    assert_ne!(assigned, default);
    default.pressure = assigned.pressure;
    assert_eq!(assigned, default, "same pressure noise becomes equal");
}
