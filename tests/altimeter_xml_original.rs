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
#[path = "support/quiet.rs"]
mod quiet;
use sdformat::*;

fn original_body() {
    let mut value = Altimeter::default();
    assert_eq!(value.vertical_position, Noise::default());
    assert_eq!(value.vertical_velocity, Noise::default());
    let mut noise = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };
    value.vertical_position = noise;
    value.vertical_velocity = noise;
    let output = value.to_xml();
    assert!(output.is_ok());
    let xml = output.unwrap();
    assert!(!xml.is_empty());
    assert!(
        value.source.is_none(),
        "immutable emission fabricates no loaded input declaration"
    );
    let loaded = load_altimeter_str(&xml, "altimeter.xml");
    assert!(loaded.is_ok());
    let mut loaded = loaded.unwrap();
    assert_eq!(loaded.vertical_position, noise);
    assert_eq!(loaded.vertical_velocity, noise);
    noise.mean = 2.3;
    loaded.vertical_position = noise;
    let output = loaded.to_xml().unwrap();
    assert!(!output.is_empty());
    assert_eq!(
        load_altimeter_str(&output, "edited.xml")
            .unwrap()
            .vertical_position,
        noise
    );
}
#[test]
fn original_altimeter_to_element() {
    original_body();
}
#[test]
fn original_altimeter_to_element_error_output() {
    quiet::assert_quiet(
        "original_altimeter_to_element_error_output",
        "SDF_ALTIMETER_XML_QUIET_CHILD",
        original_body,
    );
}
