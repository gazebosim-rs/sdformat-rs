// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
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

use sdformat::*;

fn vector(actual: [f64; 3], expected: [f64; 3]) {
    for (a, b) in actual.into_iter().zip(expected) {
        assert!((a - b).abs() <= 1e-3);
    }
    assert_eq!(actual, expected); // Supplementary exact equality.
}
fn ode_defaults(value: &OdeFriction) {
    assert_eq!(value.mu, 1.);
    assert_eq!(value.mu2, 1.);
    assert_eq!(value.slip1, 0.);
    assert_eq!(value.slip2, 0.);
    vector(value.fdir1, [0.; 3]);
}
fn bullet_defaults(value: &BulletFriction) {
    assert_eq!(value.friction, 1.);
    assert_eq!(value.friction2, 1.);
    vector(value.fdir1, [0.; 3]);
    assert_eq!(value.rolling_friction, 1.);
}
fn torsional_defaults(value: &TorsionalFriction) {
    assert_eq!(value.coefficient, 1.);
    assert!(value.use_patch_radius);
    assert_eq!(value.patch_radius, 0.);
    assert_eq!(value.surface_radius, 0.);
    assert_eq!(value.ode_slip, 0.);
}
#[test]
fn default_surface_original_assertions() {
    let value = Surface::default();
    assert!(value.source.is_none());
    assert_eq!(value.contact.collide_bitmask, 0xff);
    assert!(value.contact.category_bitmask.is_none());
    assert!(value.contact.source.is_none());
    assert!(value.friction.source.is_none());
}
#[test]
fn default_contact_original_assertions() {
    let value = Contact::default();
    assert!(value.source.is_none());
    assert_eq!(value.collide_bitmask, 0xff);
    assert!(value.category_bitmask.is_none());
    assert!(value.source.is_none());
}
#[test]
fn default_ode_original_assertions() {
    let value = OdeFriction::default();
    assert!(value.source.is_none());
    ode_defaults(&value);
}
#[test]
fn default_bullet_original_assertions() {
    let value = BulletFriction::default();
    assert!(value.source.is_none());
    bullet_defaults(&value);
}
#[test]
fn default_torsional_original_assertions() {
    let value = TorsionalFriction::default();
    assert!(value.source.is_none());
    torsional_defaults(&value);
}
#[test]
fn contact_masks_original_assertions() {
    let mut value = Contact::default();
    assert_eq!(value.collide_bitmask, 0xff); // Supplementary mutation precondition.
    value.collide_bitmask = 0x67;
    value.category_bitmask = Some(0x76);
    assert_eq!(value.collide_bitmask, 0x67);
    assert_eq!(value.category_bitmask, Some(0x76));
}
#[test]
fn set_ode_original_assertions() {
    let mut value = OdeFriction::default();
    ode_defaults(&value);
    value.mu = 0.1;
    value.mu2 = 0.2;
    value.slip1 = 3.;
    value.slip2 = 4.;
    value.fdir1 = [1., 2., 3.];
    assert_eq!(value.mu, 0.1);
    assert_eq!(value.mu2, 0.2);
    assert_eq!(value.slip1, 3.);
    assert_eq!(value.slip2, 4.);
    vector(value.fdir1, [1., 2., 3.]);
}
#[test]
fn set_bullet_original_assertions() {
    let mut value = BulletFriction::default();
    bullet_defaults(&value);
    value.friction = 0.1;
    value.friction2 = 0.2;
    value.fdir1 = [1., 2., 3.];
    value.rolling_friction = 4.;
    assert_eq!(value.friction, 0.1);
    assert_eq!(value.friction2, 0.2);
    vector(value.fdir1, [1., 2., 3.]);
    assert_eq!(value.rolling_friction, 4.);
}
#[test]
fn set_torsional_original_assertions() {
    let mut value = TorsionalFriction::default();
    torsional_defaults(&value);
    value.coefficient = 0.1;
    value.use_patch_radius = false;
    value.patch_radius = 0.2;
    value.surface_radius = 4.;
    value.ode_slip = 1.;
    assert_eq!(value.coefficient, 0.1);
    assert!(!value.use_patch_radius);
    assert_eq!(value.patch_radius, 0.2);
    assert_eq!(value.surface_radius, 4.);
    assert_eq!(value.ode_slip, 1.);
}
#[test]
fn set_friction_original_assertions() {
    let ode = OdeFriction {
        mu: 0.1,
        mu2: 0.2,
        slip1: 3.,
        slip2: 4.,
        fdir1: [1., 2., 3.],
        ..Default::default()
    };
    let bullet = BulletFriction {
        friction: 0.3,
        friction2: 0.5,
        fdir1: [2., 1., 4.],
        rolling_friction: 1.3,
        ..Default::default()
    };
    let torsional = TorsionalFriction {
        coefficient: 0.5,
        use_patch_radius: false,
        patch_radius: 0.1,
        surface_radius: 0.3,
        ode_slip: 0.2,
        ..Default::default()
    };
    let value = Friction {
        ode,
        bullet: Some(bullet),
        torsional: Some(torsional),
        ..Default::default()
    };
    assert_eq!(value.ode.mu, 0.1);
    assert_eq!(value.ode.mu2, 0.2);
    assert_eq!(value.ode.slip1, 3.);
    assert_eq!(value.ode.slip2, 4.);
    vector(value.ode.fdir1, [1., 2., 3.]);
    let bullet = value.bullet.unwrap();
    assert_eq!(bullet.friction, 0.3);
    assert_eq!(bullet.friction2, 0.5);
    vector(bullet.fdir1, [2., 1., 4.]);
    assert_eq!(bullet.rolling_friction, 1.3);
    let torsional = value.torsional.unwrap();
    assert_eq!(torsional.coefficient, 0.5);
    assert!(!torsional.use_patch_radius);
    assert_eq!(torsional.patch_radius, 0.1);
    assert_eq!(torsional.surface_radius, 0.3);
    assert_eq!(torsional.ode_slip, 0.2);
}
fn observed(surface: &Surface) -> serde_json::Value {
    let ode = &surface.friction.ode;
    serde_json::json!({
        "contact":{"collide_bitmask":surface.contact.collide_bitmask,"category_bitmask":surface.contact.category_bitmask},
        "ode":{"mu":ode.mu,"mu2":ode.mu2,"slip1":ode.slip1,"slip2":ode.slip2,"fdir1":ode.fdir1},
        "bullet":surface.friction.bullet.as_ref().map(|b| serde_json::json!({"friction":b.friction,"friction2":b.friction2,"fdir1":b.fdir1,"rolling_friction":b.rolling_friction})),
        "torsional":surface.friction.torsional.as_ref().map(|t| serde_json::json!({"coefficient":t.coefficient,"use_patch_radius":t.use_patch_radius,"patch_radius":t.patch_radius,"surface_radius":t.surface_radius,"ode_slip":t.ode_slip}))
    })
}
#[test]
fn native_surface_configuration_defaults_masks_and_errors() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("surface.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let result = load_str(
            case["xml"].as_str().unwrap(),
            case["id"].as_str().unwrap(),
            &NoResources,
        );
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{case}"
        );
        if let Some(expected) = case.get("surface") {
            let sdf = result.as_ref().unwrap();
            assert_eq!(
                observed(&sdf.models[0].links[0].collisions[0].surface),
                *expected,
                "{case}"
            );
        }
        if let Some(expected) = case.get("surface_coefficient_bits") {
            let surface = &result.as_ref().unwrap().models[0].links[0].collisions[0].surface;
            assert_eq!(
                surface
                    .friction
                    .torsional
                    .as_ref()
                    .unwrap()
                    .coefficient
                    .to_bits(),
                expected.as_u64().unwrap(),
                "{case}"
            );
        }
    }
}
fn document(surface: &str) -> Result<Sdf, Diagnostic> {
    load_str(
        &format!(
            "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><geometry><box/></geometry>{surface}</collision></link></model></sdf>"
        ),
        "surface-source",
        &NoResources,
    )
}
#[test]
fn explicit_group_provenance_and_metadata_ownership_are_preserved() {
    let sdf = document(
        "<surface>\n<contact/><friction>\n<ode/><bullet/><torsional/>\n</friction></surface>",
    )
    .unwrap();
    let value = &sdf.models[0].links[0].collisions[0].surface;
    for source in [
        &value.source,
        &value.contact.source,
        &value.friction.source,
        &value.friction.ode.source,
        &value.friction.bullet.as_ref().unwrap().source,
        &value.friction.torsional.as_ref().unwrap().source,
    ] {
        assert_eq!(source.as_ref().unwrap().uri, "surface-source");
    }
    assert_eq!(value.contact.source.as_ref().unwrap().line, 2);
    assert_eq!(value.friction.ode.source.as_ref().unwrap().line, 3);
    let absent = document("").unwrap();
    let surface = &absent.models[0].links[0].collisions[0].surface;
    assert_eq!(surface, &Surface::default());
    assert_ne!(value, surface); // Equal numeric defaults still retain explicit XML presence.
    let mut copy = value.clone();
    copy.friction.ode.mu = 0.5;
    assert_eq!(value.friction.ode.mu, 1.);
}
#[test]
fn remaining_surface_schema_is_explicit_and_native_scalar_prefixes_are_supported() {
    for xml in [
        "<surface><bounce/></surface>",
        "<surface><contact><ode/></contact></surface>",
        "<surface><friction><ode unknown='1'/></friction></surface>",
    ] {
        assert_eq!(document(xml).unwrap_err().code, ErrorCode::Unsupported);
    }
    let xml = "<surface><friction><ode><mu>0.6tail</mu></ode></friction></surface>";
    assert_eq!(
        document(xml).unwrap().models[0].links[0].collisions[0]
            .surface
            .friction
            .ode
            .mu,
        0.6
    ); // Same formerly unsupported input now retains its native value.
}
