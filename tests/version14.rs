// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{Model, Resource, ResourceResolver, Sdf, load_str};
use serde_json::Value;

struct Resolver<'a>(&'a Value);
impl ResourceResolver for Resolver<'_> {
    fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
        assert_eq!(uri, "model://child");
        Ok(Resource {
            canonical_uri: "cached/resource/model.sdf".into(),
            xml: self.0["resource"]
                .as_str()
                .expect("include resource")
                .into(),
        })
    }
}
fn vector(actual: [f64; 3], expected: &Value) {
    for (a, b) in actual.into_iter().zip(expected.as_array().unwrap()) {
        assert!((a - b.as_f64().unwrap()).abs() <= 1e-12, "{a} != {b}");
    }
}
fn axes(sdf: &Sdf, model: &Model, expected: &Value) {
    assert_eq!(
        model.joints.len(),
        expected["joints"].as_array().unwrap().len()
    );
    for (joint, row) in model
        .joints
        .iter()
        .zip(expected["joints"].as_array().unwrap())
    {
        assert_eq!(joint.name, row["name"].as_str().unwrap());
        let frame = format!("{}::{}", model.scoped_name, joint.name);
        for (axis, reference) in [&joint.axis, &joint.axis2]
            .into_iter()
            .zip(row["axes"].as_array().unwrap())
        {
            assert_eq!(axis.is_none(), reference.is_null());
            if let Some(axis) = axis {
                assert_eq!(
                    axis.expressed_in,
                    reference["expressed_in"].as_str().unwrap()
                );
                vector(axis.xyz, &reference["xyz"]);
                let resolved = sdf.resolve_axis(axis, &frame, &frame);
                assert_eq!(resolved.is_ok(), reference["errors"] == 0);
                if let Ok(xyz) = resolved {
                    vector(xyz, &reference["resolved"]);
                }
            }
        }
    }
    assert_eq!(
        model.models.len(),
        expected["models"].as_array().unwrap().len()
    );
    for (child, row) in model
        .models
        .iter()
        .zip(expected["models"].as_array().unwrap())
    {
        axes(sdf, child, row);
    }
}
#[test]
fn native_direct_and_nested_joint_axis_conversion_scopes() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("version14.json")).unwrap();
    for case in cases {
        let result = load_str(
            case["xml"].as_str().unwrap(),
            "1.4-scope.sdf",
            &Resolver(&case),
        );
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{}: {result:?}",
            case["id"]
        );
        if let Ok(sdf) = result {
            let model = sdf
                .models
                .first()
                .unwrap_or_else(|| &sdf.worlds[0].models[0]);
            axes(&sdf, model, &case["model"]);
        }
    }
}
