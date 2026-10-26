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
use sdformat::{Atmosphere, AtmosphereType};

fn configured() -> Atmosphere {
    Atmosphere {
        atmosphere_type: AtmosphereType::Adiabatic,
        temperature_kelvin: 123.23,
        temperature_gradient: -1.65,
        pressure: 76531.3,
        ..Atmosphere::default()
    }
}
fn assert_values(a: &Atmosphere, temperature: f64, gradient: f64, pressure: f64) {
    assert_eq!(a.atmosphere_type, AtmosphereType::Adiabatic);
    assert_eq!(a.temperature_kelvin.to_bits(), temperature.to_bits());
    assert_eq!(a.temperature_gradient.to_bits(), gradient.to_bits());
    assert_eq!(a.pressure.to_bits(), pressure.to_bits());
}
// Native Temperature equality and gz::math::equal use absolute <=1e-6.
// This ports the two fixed equality assertions, not a public operator facade.
fn native_equal(a: &Atmosphere, b: &Atmosphere) -> bool {
    a.atmosphere_type == b.atmosphere_type
        && (a.temperature_kelvin - b.temperature_kelvin).abs() <= 1e-6
        && (a.temperature_gradient - b.temperature_gradient).abs() <= 1e-6
        && (a.pressure - b.pressure).abs() <= 1e-6
}
#[test]
fn original_dom_atmosphere_construction() {
    assert_values(&Atmosphere::default(), 288.15, -0.0065, 101325.);
}
#[test]
fn original_dom_atmosphere_set() {
    let mut a = Atmosphere::default();
    assert_values(&a, 288.15, -0.0065, 101325.); // Supplemental initial state.
    a.atmosphere_type = AtmosphereType::Adiabatic;
    assert_eq!(a.atmosphere_type, AtmosphereType::Adiabatic);
    a.temperature_kelvin = 123.23;
    assert_eq!(a.temperature_kelvin.to_bits(), 123.23_f64.to_bits());
    a.temperature_gradient = -1.65;
    assert_eq!(a.temperature_gradient.to_bits(), (-1.65_f64).to_bits());
    a.pressure = 76531.3;
    assert_eq!(a.pressure.to_bits(), 76531.3_f64.to_bits());
}
#[test]
fn original_dom_atmosphere_move_constructor() {
    let a = configured();
    let moved = a;
    assert_values(&moved, 123.23, -1.65, 76531.3);
}
#[test]
fn original_dom_atmosphere_copy_constructor() {
    let a = configured();
    let copied = a.clone();
    assert_values(&copied, 123.23, -1.65, 76531.3);
    assert!(native_equal(&a, &copied));
}
#[test]
fn original_dom_atmosphere_move_assignment() {
    let a = configured();
    let mut moved = Atmosphere::default();
    let previous = std::mem::replace(&mut moved, a);
    assert_values(&previous, 288.15, -0.0065, 101325.); // Supplemental.
    assert_values(&moved, 123.23, -1.65, 76531.3);
}
#[test]
fn original_dom_atmosphere_copy_assignment() {
    let a = configured();
    let mut copied = Atmosphere::default();
    copied.clone_from(&a);
    assert_values(&copied, 123.23, -1.65, 76531.3);
    assert!(native_equal(&a, &copied));
}
#[test]
fn original_dom_atmosphere_copy_assignment_after_move() {
    let a1 = Atmosphere {
        temperature_kelvin: 100.,
        ..Atmosphere::default()
    };
    let a2 = Atmosphere {
        temperature_kelvin: 200.,
        ..Atmosphere::default()
    };
    let tmp = a1;
    let a1 = a2.clone();
    let a2 = tmp.clone();
    assert_eq!(a1.temperature_kelvin.to_bits(), 200_f64.to_bits());
    assert_eq!(a2.temperature_kelvin.to_bits(), 100_f64.to_bits());
}
