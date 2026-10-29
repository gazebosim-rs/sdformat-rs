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
fn double(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite() && actual >= 0. && expected >= 0.);
    assert!(actual.to_bits().abs_diff(expected.to_bits()) <= 4);
}
fn original_body() {
    let noise = Noise {
        kind: NoiseKind::Gaussian,
        mean: 1.2,
        stddev: 2.3,
        bias_mean: 4.5,
        bias_stddev: 6.7,
        precision: 8.9,
        ..Noise::default()
    };
    let value = AirPressure {
        pressure: noise,
        reference_altitude: 10.2,
        ..AirPressure::default()
    };
    let output = value.to_xml();
    assert!(output.is_ok());
    let xml = output.unwrap();
    assert!(!xml.is_empty());
    assert!(
        value.source.is_none(),
        "immutable emission fabricates no loaded input declaration"
    );
    let loaded = load_air_pressure_str(&xml, "air.xml");
    assert!(loaded.is_ok());
    let mut loaded = loaded.unwrap();
    for (actual, expected) in [
        (loaded.pressure.mean, noise.mean),
        (loaded.pressure.stddev, noise.stddev),
        (loaded.pressure.bias_mean, noise.bias_mean),
        (loaded.pressure.bias_stddev, noise.bias_stddev),
        (loaded.pressure.precision, noise.precision),
    ] {
        double(actual, expected);
    }
    double(loaded.reference_altitude, 10.2);
    loaded.reference_altitude = 111.;
    let xml = loaded.to_xml().unwrap();
    assert!(!xml.is_empty());
    double(
        load_air_pressure_str(&xml, "edited.xml")
            .unwrap()
            .reference_altitude,
        111.,
    );
}
#[test]
fn original_air_pressure_to_element() {
    original_body();
}
#[test]
fn original_air_pressure_to_element_error_output() {
    quiet::assert_quiet(
        "original_air_pressure_to_element_error_output",
        "SDF_AIR_PRESSURE_XML_QUIET_CHILD",
        original_body,
    );
}
