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

use sdformat::{Noise, NoiseKind};

#[test]
fn original_noise_construction_set_copy_and_move() {
    let mut noise = Noise::default();
    assert_eq!(noise.kind, NoiseKind::None);
    noise.kind = NoiseKind::Gaussian;
    assert_eq!(noise.kind, NoiseKind::Gaussian);

    assert_eq!(noise.mean, 0.0);
    noise.mean = 1.2;
    assert_eq!(noise.mean, 1.2);
    assert_eq!(noise.stddev, 0.0);
    noise.stddev = 2.3;
    assert_eq!(noise.stddev, 2.3);
    assert_eq!(noise.bias_mean, 0.0);
    noise.bias_mean = 4.5;
    assert_eq!(noise.bias_mean, 4.5);
    assert_eq!(noise.bias_stddev, 0.0);
    noise.bias_stddev = 6.7;
    assert_eq!(noise.bias_stddev, 6.7);
    assert_eq!(noise.precision, 0.0);
    noise.precision = 8.9;
    assert_eq!(noise.precision, 8.9);
    assert_eq!(noise.dynamic_bias_stddev, 0.0);
    noise.dynamic_bias_stddev = 9.1;
    assert_eq!(noise.dynamic_bias_stddev, 9.1);
    assert_eq!(noise.dynamic_bias_correlation_time, 0.0);
    noise.dynamic_bias_correlation_time = 19.12;
    assert_eq!(noise.dynamic_bias_correlation_time, 19.12);

    let mut noise2 = noise;
    assert_eq!(noise, noise2, "copy construction");
    let mut noise3 = Noise::default();
    assert_ne!(noise3, noise);
    noise3.clone_from(&noise);
    assert_eq!(noise, noise3, "copy assignment");

    let noise4 = std::mem::take(&mut noise);
    assert_eq!(noise2, noise4, "move construction");
    let mut noise5 = Noise::default();
    assert_ne!(noise5, noise3);
    noise5 = std::mem::take(&mut noise2);
    assert_eq!(noise3, noise5, "move assignment");

    let mut noise6 = Noise {
        mean: 0.6,
        ..Noise::default()
    };
    let mut noise7 = Noise {
        mean: 0.7,
        ..Noise::default()
    };
    let tmp = std::mem::take(&mut noise6);
    noise6 = noise7;
    noise7 = tmp;
    assert_eq!(noise6.mean, 0.7, "copy assignment after move");
    assert_eq!(noise7.mean, 0.6, "assignment from moved value");
}
