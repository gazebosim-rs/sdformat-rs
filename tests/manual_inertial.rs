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
 */
// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
#[path = "common/auto_default_density.rs"]
mod default_density;
use serde_json::Value;

fn bits(value: &Inertial) -> Vec<u64> {
    std::iter::once(value.mass)
        .chain(value.inertia)
        .chain(value.pose.transform.translation)
        .chain(value.pose.transform.rotation)
        .map(f64::to_bits)
        .collect()
}

#[test]
fn native_manual_inertial_defaults_values_and_boolean_domain() {
    let data: Value = serde_json::from_str(include_str!("manual_inertial.json")).unwrap();
    let mut admitted = 0;
    let mut manual = None;
    for row in data["cases"].as_array().unwrap() {
        let result = load_str(row["xml"].as_str().unwrap(), "manual.sdf", &NoResources);
        if row["name"] == "true_valid_box" {
            assert_eq!(row["rust_error"], "Unsupported");
            let lines: Vec<_> = row["stdout"].as_str().unwrap().lines().collect();
            assert_eq!(lines.len(), 4);
            assert_eq!(lines[0], "LOAD");
            assert_eq!(lines[2], "RESOLVE");
            assert_eq!(lines[1], lines[3]);
            let expected: Vec<u64> = lines[1]
                .strip_prefix("VALUES ")
                .unwrap()
                .split_whitespace()
                .map(|value| value.parse().unwrap())
                .collect();
            default_density::assert_complete_default_density(
                row["xml"].as_str().unwrap(),
                "manual.sdf",
                "1.12",
                "manual_box",
                None,
                &expected,
            );
            continue;
        }
        if let Some(expected) = row.get("rust_error") {
            let code = match expected.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unexpected error code"),
            };
            let error = result.unwrap_err();
            assert_eq!(error.code, code, "{}", row["name"]);
            assert_eq!(error.source.uri, "manual.sdf");
            continue;
        }
        let sdf = result.unwrap();
        let native = row["stdout"].as_str().unwrap().lines().collect::<Vec<_>>();
        assert_eq!(native[0], "LOAD");
        assert_eq!(native[2], "RESOLVE");
        assert_eq!(native[1], native[3], "{}", row["name"]);
        let native = native[1]
            .strip_prefix("VALUES ")
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse::<u64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(native.len(), 18);
        assert_eq!(&native[..2], [0, 0]);
        assert_eq!(&native[16..], [0, 0]);
        let value = &sdf.models[0].links[0].inertial;
        let actual = bits(value);
        if row["name"].as_str().unwrap().starts_with("manual:") {
            assert_eq!(actual[..10], native[2..12], "{}", row["name"]);
            // These supplemental Euler angles yield positive quaternion values.
            // Native EXPECT_DOUBLE_EQ permits four ULPs; defaults stay exact.
            for (actual, expected) in actual[10..].iter().zip(&native[12..16]) {
                assert!(f64::from_bits(*actual) >= 0. && f64::from_bits(*expected) >= 0.);
                assert!(actual.abs_diff(*expected) <= 4, "{}", row["name"]);
            }
            if row["name"] == "manual:None" {
                manual = Some(actual.clone());
            } else {
                assert_eq!(actual, *manual.as_ref().unwrap());
            }
        } else {
            assert_eq!(actual, native[2..16], "{}", row["name"]);
        }
        assert!(value.fluid_added_mass.is_none());
        let mut copied = value.clone();
        assert_eq!(bits(&copied), bits(value));
        copied.mass += 1.;
        assert_ne!(copied.mass, value.mass);
        sdf.validate_non_frame_poses().unwrap();
        admitted += 1;
    }
    assert_eq!(admitted, 38);
}

#[test]
fn original_auto_false_loader_defaults_preserve_source_and_version() {
    let xml = include_str!("fixtures/manual_inertial_original.sdf");
    assert_eq!(
        xml,
        "<?xml version=\"1.0\"?> <sdf version=\"1.11\">   <model name='shapes'>     <link name='link'>       <inertial auto='false' />     </link>   </model> </sdf>"
    );
    let sdf = load_str(xml, "auto-false-original.sdf", &NoResources).unwrap();
    assert_eq!(sdf.version, "1.11");
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!(model.name, "shapes");
    assert_eq!(model.links.len(), 1);
    assert!(model.element().unwrap().has_child("link"));
    let link = &model.links[0];
    assert_eq!(link.name, "link");
    assert_eq!(link.source.uri, "auto-false-original.sdf");
    assert_eq!(link.source.line, 1);
    assert_eq!(
        link.source.column,
        u32::try_from(xml.find("<link").unwrap() + 1).unwrap()
    );
    assert_eq!(link.inertial.mass, 1.);
    assert_eq!(link.inertial.inertia, [1., 1., 1., 0., 0., 0.]);
    assert_eq!(link.inertial.pose.transform, Transform::default());
    assert!(link.inertial.pose.relative_to.is_empty());
    assert!(link.inertial.fluid_added_mass.is_none());
    assert!(link.collisions.is_empty());
    assert!(link.visuals.is_empty());
    assert!(link.sensors.is_empty());
}

#[test]
fn explicit_manual_inertia_does_not_admit_automatic_or_unknown_domains() {
    let xml = |inertial: &str| {
        format!(
            "<sdf version='1.12'><model name='m'><link name='l'>{inertial}</link></model></sdf>"
        )
    };
    for (input, expected) in [
        (
            "<inertial auto='true'><mass>2</mass></inertial>",
            ErrorCode::Unsupported,
        ),
        ("<inertial auto='1'/>", ErrorCode::Unsupported),
        (
            "<inertial auto='false'><density>2</density></inertial>",
            ErrorCode::Unsupported,
        ),
        (
            "<inertial auto='false'><auto_inertia_params/></inertial>",
            ErrorCode::Unsupported,
        ),
        (
            "<inertial auto='false' unknown='false'/>",
            ErrorCode::Unsupported,
        ),
        (
            "<inertial auto='false'><unknown/></inertial>",
            ErrorCode::Unsupported,
        ),
        ("<inertial AUTO='false'/>", ErrorCode::Unsupported),
        ("<inertial auto='00'/>", ErrorCode::InvalidValue),
        ("<inertial auto='false&#13;'/>", ErrorCode::InvalidValue),
        (
            "<inertial auto='false'><mass>0</mass></inertial>",
            ErrorCode::InvalidValue,
        ),
        (
            "<inertial auto='false'><mass>nan</mass></inertial>",
            ErrorCode::InvalidValue,
        ),
        (
            "<inertial auto='false'><inertia><ixx>-1</ixx></inertia></inertial>",
            ErrorCode::InvalidValue,
        ),
        ("<inertial auto='false' auto='false'/>", ErrorCode::Xml),
        (
            "<inertial auto='false'/><inertial auto='false'/>",
            ErrorCode::InvalidValue,
        ),
    ] {
        assert_eq!(
            load_str(&xml(input), "unsupported.sdf", &NoResources)
                .unwrap_err()
                .code,
            expected,
            "{input}"
        );
    }
}
