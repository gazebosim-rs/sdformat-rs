// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, load_str};

#[test]
fn native_axis_limits_preserve_loaded_values_without_ordering_policy() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("axis_limits.json")).unwrap();
    for row in cases.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let parsed = load_str(row["xml"].as_str().unwrap(), "limits.sdf", &NoResources);
        let tokens = row["tokens"].as_array().unwrap();
        if row.get("gap").is_some() {
            assert!(matches!(id, "nonfinite_lower_inf" | "nonfinite_upper_nan"));
            assert_eq!(tokens[0], "0");
            assert_eq!(parsed.unwrap_err().code, ErrorCode::InvalidValue);
            continue;
        }
        if tokens[0] != "0" {
            let code = if row["rust_error"] == "Missing" {
                ErrorCode::Missing
            } else {
                ErrorCode::InvalidValue
            };
            assert_eq!(parsed.unwrap_err().code, code, "{id}");
            continue;
        }
        let sdf = parsed.unwrap_or_else(|error| panic!("{id}: {error}"));
        let joint = &sdf.models[0].joints[0];
        let mut expected = tokens.iter().skip(1);
        for index in 0..2 {
            let present = expected.next().unwrap() == "1";
            let axis = joint.axis(index);
            assert_eq!(axis.is_some(), present, "{id}, axis {index}");
            if let Some(axis) = axis {
                for value in [axis.lower, axis.upper, axis.effort, axis.velocity] {
                    let bits: u64 = expected.next().unwrap().as_str().unwrap().parse().unwrap();
                    assert_eq!(value.to_bits(), bits, "{id}, axis {index}");
                }
            }
        }
        assert!(expected.next().is_none());
    }
}

#[test]
fn exact_former_rust_only_ordering_rejections_are_native_positive_values() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("axis_limits.json")).unwrap();
    for (id, index) in [
        ("former_semantics_rejection", 0),
        ("former_visuals_axis2_rejection", 1),
    ] {
        let row = cases
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap();
        let sdf = load_str(
            row["xml"].as_str().unwrap(),
            "former-rejection.sdf",
            &NoResources,
        )
        .unwrap();
        let axis = sdf.models[0].joints[0].axis(index).unwrap();
        assert_eq!((axis.lower, axis.upper), (2., 1.));
        assert_eq!(axis.xyz, [0., 0., 1.]);
        assert_eq!((axis.effort, axis.velocity), (f64::INFINITY, f64::INFINITY));
    }
}
