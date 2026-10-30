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
use sdformat::*;
#[path = "support/quiet.rs"]
mod quiet;
fn double_eq(actual: f64, expected: f64) {
    // The selected original's scalars are positive and finite.
    assert!(actual.to_bits().abs_diff(expected.to_bits()) <= 4);
}
fn original() {
    let atmosphere = Atmosphere {
        atmosphere_type: AtmosphereType::Adiabatic,
        temperature_kelvin: 123.,
        temperature_gradient: 1.34,
        pressure: 2.65,
        ..Default::default()
    };
    let xml = atmosphere.to_xml().unwrap();
    assert!(!xml.is_empty());
    let loaded = load_atmosphere_str(&xml, "atmosphere-roundtrip.xml").unwrap();
    assert!((atmosphere.temperature_kelvin - loaded.temperature_kelvin).abs() <= 1e-6);
    double_eq(atmosphere.temperature_gradient, loaded.temperature_gradient);
    double_eq(atmosphere.pressure, loaded.pressure);
}
#[test]
fn original_atmosphere_to_element() {
    original();
}
#[test]
fn original_atmosphere_to_element_error_output() {
    quiet::assert_quiet(
        "original_atmosphere_to_element_error_output",
        "SDF_ATMOSPHERE_QUIET",
        original,
    );
}
