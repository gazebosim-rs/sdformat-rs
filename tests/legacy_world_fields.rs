/*
 * Copyright 2015 Open Source Robotics Foundation
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
 */

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, Source, load_str};

#[test]
fn pinned_legacy_world_field_controls() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("legacy_world_fields.json")).unwrap();
    for row in data["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        let result = load_str(xml, "legacy.sdf", &NoResources);
        if let Some(error) = row.get("rust_error") {
            let expected = match error.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown control error"),
            };
            let actual = result.unwrap_err();
            assert_eq!(actual.code, expected, "{name}: {actual}");
            assert_eq!(actual.source.uri, "legacy.sdf");
            continue;
        }
        let sdf = result.unwrap_or_else(|error| panic!("{name}: {error}"));
        let native = row["native"]["stdout"].as_str().unwrap();
        assert_eq!(native.lines().next(), Some("ERRORS"), "{name}");
        let bits = native
            .lines()
            .find_map(|line| line.strip_prefix("VALUES "))
            .unwrap()
            .split_whitespace()
            .map(|token| token.parse::<u64>().unwrap())
            .collect::<Vec<_>>();
        let world = &sdf.worlds[0];
        assert_eq!(
            world
                .gravity
                .into_iter()
                .chain(world.magnetic_field)
                .map(f64::to_bits)
                .collect::<Vec<_>>(),
            bits,
            "{name}"
        );
        let count = native
            .lines()
            .find_map(|line| line.strip_prefix("PHYSICS "))
            .unwrap()
            .parse::<usize>()
            .unwrap();
        assert_eq!(world.physics.len(), count, "{name}");
        assert!(sdf.models.is_empty());
        sdf.validate_non_frame_poses().unwrap();
    }
}

fn original(xml: &str, version: &str) {
    let sdf = load_str(xml, "original.sdf", &NoResources).unwrap();
    assert_eq!(sdf.version, version);
    assert!(sdf.models.is_empty());
    assert_eq!(sdf.worlds.len(), 1);
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "default");
    assert_eq!(world.physics.len(), 1);
    let physics = &world.physics[0];
    assert_eq!(physics.name, "default_physics");
    assert_eq!(physics.engine_type, "ode");
    // These retain native Vector3d equality; exact bits are supplemental.
    for (actual, expected) in world
        .gravity
        .into_iter()
        .chain(world.magnetic_field)
        .zip([1.0_f64, 0., -9.8, 1., 2., 3.])
    {
        assert!((actual - expected).abs() <= 1e-3);
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
    assert_eq!(
        world.source,
        Source {
            uri: "original.sdf".into(),
            line: 4,
            column: 3
        }
    );
    assert_eq!(
        physics.source,
        Some(Source {
            uri: "original.sdf".into(),
            line: 5,
            column: 5
        })
    );
    assert_eq!(
        physics.xml.as_deref(),
        Some(
            "<physics type=\"ode\">\n      <gravity>1 0 -9.8</gravity>\n      <magnetic_field>1 2 3</magnetic_field>\n    </physics>"
        )
    );
    // No converted Element tree or output-version API is claimed here.
    sdf.validate_non_frame_poses().unwrap();
}

#[test]
fn original_parser_string_converter_typed_world_values() {
    original(include_str!("fixtures/legacy_world_fields_15.sdf"), "1.5");
}

#[test]
fn original_parser_string_converter_from14_typed_world_values() {
    original(include_str!("fixtures/legacy_world_fields_14.sdf"), "1.4");
}

#[test]
fn legacy_fields_keep_input_source_and_physics_snapshot() {
    let xml = "<sdf version='1.5'><world name='w'>\n<physics name='p' type='ode'>\n<gravity>1 2 3</gravity>\n<magnetic_field>4 5 6</magnetic_field></physics></world></sdf>";
    let mut sdf = load_str(xml, "input.sdf", &NoResources).unwrap();
    let physics = sdf.worlds[0].physics[0].clone();
    sdf.worlds[0].gravity = [7.; 3];
    drop(sdf);
    assert_eq!(
        physics.source,
        Some(Source {
            uri: "input.sdf".into(),
            line: 2,
            column: 1
        })
    );
    assert_eq!(
        physics.xml.as_deref(),
        Some(
            "<physics name='p' type='ode'>\n<gravity>1 2 3</gravity>\n<magnetic_field>4 5 6</magnetic_field></physics>"
        )
    );
    let invalid = xml.replace("1 2 3", "bad");
    let error = load_str(&invalid, "bad.sdf", &NoResources).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!(
        error.source,
        Source {
            uri: "bad.sdf".into(),
            line: 3,
            column: 1
        }
    );
}
