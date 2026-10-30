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
#![cfg(all(target_os = "linux", target_env = "gnu"))]
use sdformat::*;
#[path = "support/quiet.rs"]
mod quiet;

fn export(name: &str, xml: &str) {
    if let Ok(dir) = std::env::var("SDF_COLLISION_XML_EXPORT") {
        std::fs::write(std::path::Path::new(&dir).join(format!("{name}.xml")), xml).unwrap();
    }
}
#[test]
fn original_collision_to_element() {
    // Collision_TEST.cc uses default EMPTY geometry and a real Surface value.
    let mut collision =
        load_collision_str("<collision name='my-collision'/>", "setup.xml").unwrap();
    collision.pose.transform.translation = [1., 2., 3.];
    collision.pose.transform.rotation =
        glam::DQuat::from_euler(glam::EulerRot::ZYX, 0.3, 0.2, 0.1).to_array();
    collision.geometry = Geometry::Empty;
    collision.surface.contact.collide_bitmask = 123;
    collision.surface.friction.ode.mu = 1.23;
    let xml = collision.to_xml().unwrap();
    assert!(!xml.contains("<density>"));
    export("original", &xml);
    let loaded = load_collision_str(&xml, "output.xml").unwrap();
    assert_eq!(loaded.density, None);
    assert_eq!(collision.name, loaded.name);
    for (a, b) in collision
        .pose
        .transform
        .translation
        .iter()
        .chain(&collision.pose.transform.rotation)
        .zip(
            loaded
                .pose
                .transform
                .translation
                .iter()
                .chain(&loaded.pose.transform.rotation),
        )
    {
        assert!((a - b).abs() <= 1e-6);
    }
    assert_eq!(loaded.geometry, Geometry::Empty);
    assert_eq!(loaded.surface.contact.collide_bitmask, 123);
    assert_eq!(loaded.surface.friction.ode.mu, 1.23);
    collision.density = Some(1234.5);
    let xml_with_density = collision.to_xml().unwrap();
    assert!(xml_with_density.contains("<density>1234.5</density>"));
    export("original_density", &xml_with_density);
    let _collision3 = load_collision_str(&xml, "output.xml").unwrap();
    assert_eq!(collision.density(), 1234.5);
    assert_eq!(
        load_collision_str(&xml_with_density, "output.xml")
            .unwrap()
            .density,
        Some(1234.5)
    );
}
#[test]
fn original_collision_to_element_error_output() {
    quiet::assert_quiet(
        "original_collision_to_element_error_output",
        "SDF_COLLISION_QUIET_CHILD",
        || {
            let mut collision =
                load_collision_str("<collision name='my-collision'/>", "setup.xml").unwrap();
            collision.pose.transform.translation = [1., 2., 3.];
            collision.pose.transform.rotation =
                glam::DQuat::from_euler(glam::EulerRot::ZYX, 0.3, 0.2, 0.1).to_array();
            collision.geometry = Geometry::Empty;
            collision.surface.contact.collide_bitmask = 123;
            collision.surface.friction.ode.mu = 1.23;
            let xml = collision.to_xml().expect("native output errors empty");
            assert!(!xml.is_empty());
            let loaded = load_collision_str(&xml, "output.xml").expect("native load errors empty");
            assert_eq!(collision.name, loaded.name);
            for (a, b) in collision
                .pose
                .transform
                .translation
                .iter()
                .chain(&collision.pose.transform.rotation)
                .zip(
                    loaded
                        .pose
                        .transform
                        .translation
                        .iter()
                        .chain(&loaded.pose.transform.rotation),
                )
            {
                assert!((a - b).abs() <= 1e-6);
            }
            assert_eq!(loaded.geometry, Geometry::Empty);
            assert_eq!(loaded.surface.contact.collide_bitmask, 123);
            assert_eq!(loaded.surface.friction.ode.mu, 1.23);
        },
    );
}
#[test]
fn density_presence_bits_and_rejections() {
    let mut value = load_collision_str("<collision name='c'/>", "input").unwrap();
    for density in [
        None,
        Some(0.),
        Some(-0.),
        Some(-1.),
        Some(1000.),
        Some(f64::MAX),
        Some(f64::MIN_POSITIVE),
    ] {
        value.density = density;
        let output = value.to_xml().unwrap();
        let loaded = load_collision_str(&output, "output").unwrap();
        assert_eq!(density.map(f64::to_bits), loaded.density.map(f64::to_bits));
        assert_eq!(loaded.geometry, Geometry::Empty);
    }
    for density in [f64::from_bits(1), f64::NAN, f64::INFINITY] {
        value.density = Some(density);
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        assert_eq!(value.density.unwrap().to_bits(), density.to_bits());
    }
}
#[test]
fn writer_and_reader_reject_unrepresentable_states() {
    let mut value = load_collision_str("<collision name='c'/>", "input").unwrap();
    for name in ["", "world", "__bad__", "a::b", " bad", "bad\n"] {
        value.name = name.into();
        assert!(value.to_xml().is_err());
    }
    value.name = "c".into();
    value.pose.transform.translation[0] = f64::from_bits(1);
    assert!(value.to_xml().is_err());
    value.pose.transform.translation[0] = f64::INFINITY;
    assert!(value.to_xml().is_err());
    for xml in [
        "<collision/>",
        "<collision name='c'><unknown/></collision>",
        "<collision name='c'><density>1</density><density>2</density></collision>",
    ] {
        let error = load_collision_str(xml, "caller.xml").unwrap_err();
        assert_eq!(error.source.uri, "caller.xml");
        assert_eq!(error.source.line, 1);
    }
    assert_eq!(
        load_collision_str("<visual/>", "x").unwrap_err().code,
        ErrorCode::Xml
    );
    assert_eq!(
        load_collision_str("<collision", "x").unwrap_err().code,
        ErrorCode::Xml
    );
}

#[test]
fn delegated_geometry_and_surface_roundtrip_and_reject_invalid() {
    let mut value = load_collision_str("<collision name='c'><geometry><box><size>2 3 4</size></box></geometry><surface><contact><collide_bitmask>123</collide_bitmask></contact><friction><ode><mu>1.23</mu></ode></friction></surface></collision>", "x").unwrap();
    let loaded = load_collision_str(&value.to_xml().unwrap(), "y").unwrap();
    assert_eq!(loaded.geometry, value.geometry);
    assert_eq!(loaded.surface.contact.collide_bitmask, 123);
    assert_eq!(loaded.surface.friction.ode.mu, 1.23);
    value.geometry = Geometry::Sphere { radius: -1. };
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    value.geometry = Geometry::Empty;
    value.surface.friction.ode.mu = -1.;
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
}

#[test]
fn native_constructor_fixture_is_loaded() {
    let value = load_collision_str(
        include_str!("fixtures/collision_to_element_native.xml"),
        "native.xml",
    )
    .unwrap();
    assert_eq!(value.name, "my-collision");
    assert_eq!(value.geometry, Geometry::Empty);
    assert_eq!(value.density, None);
    assert_eq!(value.pose.transform.translation, [1., 2., 3.]);
    let expected = glam::DQuat::from_euler(glam::EulerRot::ZYX, 0.3, 0.2, 0.1).to_array();
    for (a, b) in value.pose.transform.rotation.iter().zip(expected) {
        assert!((a - b).abs() <= 1e-6);
    }
    assert_eq!(value.surface.contact.collide_bitmask, 123);
    assert_eq!(value.surface.friction.ode.mu, 1.23);
}
