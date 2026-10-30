/*
 * Copyright (C) 2020 Open Source Robotics Foundation
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
    assert!(
        actual.to_bits().abs_diff(expected.to_bits()) <= 4,
        "{actual} != {expected}"
    );
}
fn vector(actual: [f64; 3], expected: [f64; 3]) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!((actual - expected).abs() <= 1e-3);
    }
}
fn check(all: bool) {
    let mut value = Surface::default();
    value.contact.collide_bitmask = 0x12;
    value.contact.category_bitmask = Some(0x21);
    value.friction.ode = OdeFriction {
        mu: 0.1,
        mu2: 0.2,
        slip1: 3.,
        slip2: 4.,
        fdir1: [1., 2., 3.],
        ..Default::default()
    };
    if all {
        value.friction.bullet = Some(BulletFriction {
            friction: 0.3,
            friction2: 0.5,
            fdir1: [2., 1., 4.],
            rolling_friction: 1.3,
            ..Default::default()
        });
        value.friction.torsional = Some(TorsionalFriction {
            coefficient: 0.5,
            use_patch_radius: false,
            patch_radius: 0.1,
            surface_radius: 0.3,
            ode_slip: 0.2,
            ..Default::default()
        });
    }
    let xml = value.to_xml().unwrap();
    assert!(!xml.is_empty());
    let surface = load_surface_str(&xml, "surface-original.xml").unwrap();
    assert_eq!(surface.contact.collide_bitmask, 0x12);
    assert_eq!(surface.contact.category_bitmask, Some(0x21));
    double_eq(surface.friction.ode.mu, 0.1);
    double_eq(surface.friction.ode.mu2, 0.2);
    double_eq(surface.friction.ode.slip1, 3.);
    double_eq(surface.friction.ode.slip2, 4.);
    vector(surface.friction.ode.fdir1, [1., 2., 3.]);
    if all {
        let bullet = surface.friction.bullet.as_ref().unwrap();
        double_eq(bullet.friction, 0.3);
        double_eq(bullet.friction2, 0.5);
        vector(bullet.fdir1, [2., 1., 4.]);
        double_eq(bullet.rolling_friction, 1.3);
        let torsional = surface.friction.torsional.as_ref().unwrap();
        double_eq(torsional.coefficient, 0.5);
        assert!(!torsional.use_patch_radius);
        double_eq(torsional.patch_radius, 0.1);
        double_eq(torsional.surface_radius, 0.3);
        double_eq(torsional.ode_slip, 0.2);
    }
}
#[test]
fn original_surface_to_element() {
    check(true);
}
#[test]
fn original_surface_to_element_error_output() {
    quiet::assert_quiet(
        "original_surface_to_element_error_output",
        "SDF_SURFACE_QUIET_CHILD",
        || check(false),
    );
}
