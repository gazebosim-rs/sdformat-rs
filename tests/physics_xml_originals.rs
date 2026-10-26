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
// Fixed native value observations; Element and console remain unported.
use sdformat::{Diagnostic, ErrorCode, NoResources, Physics, load_str};

fn configured() -> Physics {
    Physics {
        name: "my-bullet-engine".into(),
        is_default: true,
        engine_type: "bullet".into(),
        max_step_size: 0.1,
        real_time_factor: 20.4,
        max_contacts: 42,
        ..Physics::default()
    }
}

fn load_profile(xml: &str) -> Result<Physics, Diagnostic> {
    let document = format!("<sdf version='1.12'><world name='w'>{xml}</world></sdf>");
    let sdf = load_str(&document, "profile.sdf", &NoResources)?;
    Ok(sdf.worlds[0].physics[0].clone())
}

fn assert_original_values(original: &Physics, loaded: &Physics) {
    assert_eq!(loaded.name, original.name);
    assert_eq!(loaded.is_default, original.is_default);
    assert_eq!(loaded.engine_type, original.engine_type);
    assert_eq!(
        loaded.max_step_size.to_bits(),
        original.max_step_size.to_bits()
    );
    assert_eq!(
        loaded.real_time_factor.to_bits(),
        original.real_time_factor.to_bits()
    );
    assert_eq!(loaded.max_contacts, original.max_contacts);
}

#[test]
fn partial_dom_physics_to_element_values() {
    let physics = configured();
    let xml = physics.to_xml().unwrap();
    let physics2 = load_profile(&xml).unwrap();
    assert_original_values(&physics, &physics2);
}

#[test]
fn partial_dom_physics_to_element_error_output_values() {
    let physics = configured();
    let output = physics.to_xml();
    assert!(output.is_ok()); // Owned successful Result maps the empty emit errors.
    let loaded = load_profile(&output.unwrap());
    assert!(loaded.is_ok()); // Empty owned load diagnostic.
    assert_original_values(&physics, &loaded.unwrap());
    // Native Element pointers and redirected console remain unported.
}

#[test]
fn supplemental_physics_xml_escapes_and_finite_metadata_roundtrip() {
    let mut physics = Physics {
        name: "p & \"<>'".into(),
        engine_type: "custom & \"<>'".into(),
        ..Physics::default()
    };
    for scalar in [0., -0., -1., f64::MIN_POSITIVE] {
        physics.max_step_size = scalar;
        physics.real_time_factor = scalar;
        physics.real_time_update_rate = scalar;
        for contacts in [i32::MIN, i32::MAX] {
            physics.max_contacts = contacts;
            let xml = physics.to_xml().unwrap();
            assert!(xml.contains("&amp;"));
            assert!(xml.contains("&quot;"));
            let loaded = load_profile(&xml).unwrap();
            assert_original_values(&physics, &loaded);
            assert_eq!(loaded.real_time_update_rate.to_bits(), scalar.to_bits());
        }
    }
}

#[test]
fn supplemental_physics_xml_rejects_nonroundtrippable_attributes() {
    for name in [
        "",
        " world",
        "world",
        "__bad__",
        "a::b",
        "a\nb",
        "a\u{1}",
        "a\u{ffff}",
    ] {
        let physics = Physics {
            name: name.into(),
            ..configured()
        };
        assert_eq!(physics.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    for engine in ["", " ode", "ode ", "o\td", "o\rd", "o\u{fffe}"] {
        let physics = Physics {
            engine_type: engine.into(),
            ..configured()
        };
        assert_eq!(physics.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
}

#[test]
fn supplemental_physics_xml_rejects_numeric_range_failures() {
    let setters: [fn(&mut Physics, f64); 3] = [
        |p, v| p.max_step_size = v,
        |p, v| p.real_time_factor = v,
        |p, v| p.real_time_update_rate = v,
    ];
    for set in setters {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e-310] {
            let mut physics = configured();
            set(&mut physics, value);
            assert_eq!(physics.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
}
