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
use sdformat::*;
#[test]
fn original_navsat_to_element() {
    let horizontal_position = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };
    let vertical_position = Noise {
        mean: 2.2,
        ..horizontal_position
    };
    let horizontal_velocity = Noise {
        mean: 3.2,
        ..horizontal_position
    };
    let vertical_velocity = Noise {
        mean: 4.2,
        ..horizontal_position
    };
    let value = NavSat {
        horizontal_position,
        vertical_position,
        horizontal_velocity,
        vertical_velocity,
        ..NavSat::default()
    };
    let output = value.to_xml();
    assert!(output.is_ok());
    let xml = output.unwrap();
    assert!(!xml.is_empty());
    assert!(
        value.source.is_none(),
        "immutable emission fabricates no loaded input declaration"
    );
    let loaded = load_navsat_str(&xml, "navsat.xml");
    assert!(loaded.is_ok());
    let loaded = loaded.unwrap();
    assert_eq!(loaded.horizontal_position, horizontal_position);
    assert_eq!(loaded.vertical_position, vertical_position);
    assert_eq!(loaded.horizontal_velocity, horizontal_velocity);
    assert_eq!(loaded.vertical_velocity, vertical_velocity);
}
