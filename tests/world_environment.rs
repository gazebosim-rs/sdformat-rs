// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, load_str};
use serde_json::Value;
#[path = "support/world_environment_projection.rs"]
mod environment;

fn data() -> Value {
    serde_json::from_str(include_str!("world_environment.json")).unwrap()
}
#[test]
fn world_vectors_match_native_defaults_values_and_explicit_policy_boundaries() {
    for row in data()["cases"].as_array().unwrap() {
        let value = load_str(
            row["xml"].as_str().unwrap(),
            "environment.sdf",
            &NoResources,
        );
        if let Some(gap) = row.get("gap_code") {
            let code = match gap.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "DuplicateName" => ErrorCode::DuplicateName,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown policy boundary"),
            };
            assert_eq!(value.unwrap_err().code, code, "{}", row["id"]);
        } else if row["native"]["stdout"]
            .as_str()
            .unwrap()
            .starts_with("ROOT\n")
        {
            let sdf = value.unwrap_or_else(|e| panic!("{}: {e:?}", row["id"]));
            assert_eq!(sdf.worlds.len(), 1);
            environment::compare(&sdf.worlds[0], &row["native"]);
        } else {
            assert_eq!(
                value.unwrap_err().code,
                ErrorCode::InvalidValue,
                "{}",
                row["id"]
            );
        }
    }
}
#[test]
fn original_dom_world_load_vector_assertions_on_unmodified_subtrees() {
    let sdf = load_str(
        include_str!("fixtures/world_environment_original.sdf"),
        "world_complete.sdf",
        &NoResources,
    )
    .unwrap();
    assert_eq!(sdf.worlds.len(), 1);
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "default");
    for (actual, expected) in [
        (world.wind_linear_velocity, [4., 5., 6.]),
        (world.gravity, [1., 2., 3.]),
        (world.magnetic_field, [-1., 0.5, 10.]),
    ] {
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!((actual - expected).abs() <= 1e-3);
        }
    }
    environment::compare(world, &data()["fixture"]["full"]);
}
#[test]
fn malformed_vectors_keep_the_callers_location_and_unrelated_features_unsupported() {
    for (xml, line, column) in [
        (
            "<sdf version='1.12'><world name='w'>\n<magnetic_field/></world></sdf>",
            2,
            1,
        ),
        (
            "<sdf version='1.12'><world name='w'>\n<wind><linear_velocity>1 2</linear_velocity></wind></world></sdf>",
            2,
            7,
        ),
    ] {
        let error = load_str(xml, "bad-environment.sdf", &NoResources).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidValue);
        assert_eq!(
            error.source,
            sdformat::Source {
                uri: "bad-environment.sdf".into(),
                line,
                column
            }
        );
    }
    for payload in ["<scene/>", "<gui/>", "<projector/>"] {
        let xml = format!("<sdf version='1.12'><world name='w'><wind/>{payload}</world></sdf>");
        assert_eq!(
            load_str(&xml, "unsupported.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}
