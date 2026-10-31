/*
 * Copyright 2017 Open Source Robotics Foundation
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
use sdformat::{ErrorCode, NoResources, SphericalSurface, WorldFrameOrientation, load_str};
use serde_json::Value;

fn native_metadata(stdout: &str) -> String {
    stdout
        .lines()
        .map(|line| {
            if line.starts_with("METRES ") {
                format!("METRES {}", line.split_whitespace().nth(1).unwrap())
            } else {
                line.into()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
fn observations(xml: &str, uri: &str) -> Result<String, sdformat::Diagnostic> {
    let sdf = load_str(xml, uri, &NoResources)?;
    let world = &sdf.worlds[0];
    assert_eq!(world.source.uri, uri);
    assert!(world.extensions.is_empty());
    assert!(world.models.is_empty());
    assert!(world.lights.is_empty());
    let Some(value) = &world.spherical_coordinates else {
        return Ok("ERRORS 0\nCOORDS 0\n".into());
    };
    assert_eq!(value.source.uri, uri);
    assert_eq!(value.orientation, WorldFrameOrientation::EastNorthUp);
    let surface = match value.surface {
        SphericalSurface::EarthWgs84 => 1,
        SphericalSurface::MoonScs => 2,
    };
    let angles = [
        value.latitude_radians,
        value.longitude_radians,
        value.heading_radians,
    ];
    let radians = angles.map(f64::to_bits);
    let degrees = angles.map(|a| (a * 180. / std::f64::consts::PI).to_bits());
    Ok(format!(
        "ERRORS 0\nCOORDS 1\nSURFACE {surface}\nRADIANS {} {} {}\nDEGREES {} {} {}\nMETRES {}\n",
        radians[0],
        radians[1],
        radians[2],
        degrees[0],
        degrees[1],
        degrees[2],
        value.elevation.to_bits()
    ))
}
#[test]
fn spherical_metadata_matches_pinned_native_observations() {
    let data: Value = serde_json::from_str(include_str!("world_spherical.json")).unwrap();
    for row in data["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let result = observations(row["xml"].as_str().unwrap(), "spherical.sdf");
        match row["rust_policy"].as_str().unwrap() {
            "accepted" => assert_eq!(
                result.unwrap_or_else(|e| panic!("{name}: {e:?}")),
                native_metadata(row["stdout"].as_str().unwrap()),
                "{name}"
            ),
            policy => {
                let code = match policy {
                    "Missing" => ErrorCode::Missing,
                    "Unsupported" => ErrorCode::Unsupported,
                    "InvalidValue" => ErrorCode::InvalidValue,
                    _ => panic!("unknown policy {policy}"),
                };
                assert_eq!(result.unwrap_err().code, code, "{name}");
            }
        }
    }
}
#[test]
fn original_world_load_spherical_assertions_retain_native_double_tolerance() {
    let xml = include_str!("fixtures/world_spherical_original.sdf");
    let sdf = load_str(xml, "world_complete.sdf#spherical", &NoResources).unwrap();
    assert_eq!(sdf.worlds.len(), 1);
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "default");
    let value = world.spherical_coordinates.as_ref().unwrap();
    assert_eq!(value.surface, SphericalSurface::EarthWgs84);
    for (radians, expected) in [
        (value.latitude_radians, -22.9_f64),
        (value.longitude_radians, -43.2),
        (value.heading_radians, 90.),
    ] {
        let degrees = radians * 180. / std::f64::consts::PI;
        assert!(degrees.to_bits().abs_diff(expected.to_bits()) <= 4);
    }
    assert!(value.elevation.to_bits().abs_diff(100_f64.to_bits()) <= 4);
    assert_eq!((value.source.line, value.source.column), (3, 5));
    assert_eq!(value.source.uri, "world_complete.sdf#spherical");
    let data: Value = serde_json::from_str(include_str!("world_spherical.json")).unwrap();
    assert_eq!(
        observations(xml, "world_complete.sdf#spherical").unwrap(),
        native_metadata(data["full_original"]["stdout"].as_str().unwrap())
    );
}
#[test]
fn spherical_metadata_owns_its_values_and_input_source() {
    let mut sdf = {
        let xml = String::from(
            "<sdf version='1.12'><world name='w'><spherical_coordinates/></world></sdf>",
        );
        load_str(&xml, "owned.sdf", &NoResources).unwrap()
    };
    let value = sdf.worlds[0].spherical_coordinates.as_ref().unwrap();
    assert_eq!((value.source.line, value.source.column), (1, 37));
    assert_eq!(&value.clone(), value);
    let mut copy = value.clone();
    copy.source.uri.clear();
    copy.elevation = 8.;
    assert_eq!(value.source.uri, "owned.sdf");
    assert_eq!(value.elevation, 0.);
    assert_ne!(&copy, value);
    sdf.worlds[0].spherical_coordinates = None;
    assert_eq!(copy.elevation, 8.);
    assert!(format!("{copy:?}").contains("EarthWgs84"));
}
#[test]
fn spherical_payload_validation_and_angle_overflow_remain_explicit() {
    for (payload, code) in [
        (
            "<spherical_coordinates><latitude_deg extra='x'/></spherical_coordinates>",
            ErrorCode::Missing,
        ),
        (
            "<spherical_coordinates><heading_deg><unknown/></heading_deg></spherical_coordinates>",
            ErrorCode::Missing,
        ),
        (
            "<spherical_coordinates/><spherical_coordinates/>",
            ErrorCode::InvalidValue,
        ),
        (
            "<spherical_coordinates><latitude_deg extra='x'>1</latitude_deg></spherical_coordinates>",
            ErrorCode::Unsupported,
        ),
        (
            "<spherical_coordinates><surface_model><x/></surface_model></spherical_coordinates>",
            ErrorCode::Unsupported,
        ),
        (
            "<spherical_coordinates><world_frame_orientation extra='x'>ENU</world_frame_orientation></spherical_coordinates>",
            ErrorCode::Unsupported,
        ),
        (
            "<spherical_coordinates><latitude_deg>1e308</latitude_deg></spherical_coordinates>",
            ErrorCode::InvalidValue,
        ),
        (
            "<spherical_coordinates><longitude_deg>bad</longitude_deg></spherical_coordinates>",
            ErrorCode::InvalidValue,
        ),
        (
            "<spherical_coordinates><elevation>bad</elevation></spherical_coordinates>",
            ErrorCode::InvalidValue,
        ),
        (
            "<spherical_coordinates><heading_deg>bad</heading_deg></spherical_coordinates>",
            ErrorCode::InvalidValue,
        ),
    ] {
        let xml = format!("<sdf version='1.12'><world name='w'>{payload}</world></sdf>");
        assert_eq!(
            load_str(&xml, "spherical.sdf", &NoResources)
                .unwrap_err()
                .code,
            code,
            "{payload}"
        );
    }
}
