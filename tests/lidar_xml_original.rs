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
use sdformat::*;
fn double(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite() && actual >= 0. && expected >= 0.);
    assert!(actual.to_bits().abs_diff(expected.to_bits()) <= 4);
}
#[test]
fn original_lidar_to_element() {
    let noise = Noise {
        mean: 6.5,
        stddev: 3.79,
        ..Noise::default()
    };
    let value = Lidar {
        horizontal: LidarScan {
            samples: 123,
            resolution: 0.45,
            min_angle: 0.67,
            max_angle: 0.89,
        },
        vertical: LidarScan {
            samples: 98,
            resolution: 0.76,
            min_angle: 0.54,
            max_angle: 0.321,
        },
        range_min: 1.2,
        range_max: 3.4,
        range_resolution: 5.6,
        visibility_mask: 123,
        noise,
        ..Lidar::default()
    };
    let output = value.to_xml();
    assert!(output.is_ok());
    let xml = output.unwrap();
    assert!(!xml.is_empty());
    assert!(
        value.source.is_none(),
        "immutable emission fabricates no loaded input declaration"
    );
    let loaded = load_lidar_str(&xml, "lidar.xml");
    assert!(loaded.is_ok());
    let mut loaded = loaded.unwrap();
    assert_eq!(loaded.horizontal.samples, 123);
    assert_eq!(loaded.vertical.samples, 98);
    for (actual, expected) in [
        (loaded.horizontal.resolution, 0.45),
        (loaded.horizontal.min_angle, 0.67),
        (loaded.horizontal.max_angle, 0.89),
        (loaded.vertical.resolution, 0.76),
        (loaded.vertical.min_angle, 0.54),
        (loaded.vertical.max_angle, 0.321),
        (loaded.range_min, 1.2),
        (loaded.range_max, 3.4),
        (loaded.range_resolution, 5.6),
    ] {
        double(actual, expected);
    }
    assert_eq!(loaded.visibility_mask, 123);
    assert_eq!(loaded.noise, noise);
    loaded.horizontal.samples = 111;
    let xml = loaded.to_xml().unwrap();
    assert!(!xml.is_empty());
    assert_eq!(
        load_lidar_str(&xml, "edited.xml")
            .unwrap()
            .horizontal
            .samples,
        111
    );
}
