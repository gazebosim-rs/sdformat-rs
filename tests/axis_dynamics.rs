// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, load_str};
use serde_json::Value;

#[test]
fn native_axis_dynamics_defaults_values_and_errors() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("axis_dynamics.json")).unwrap();
    for row in rows {
        let result = load_str(row["xml"].as_str().unwrap(), "joint.sdf", &NoResources);
        let tokens = row["tokens"].as_array().unwrap();
        if row.get("gap").is_some() {
            assert_eq!(tokens[0], "0");
            let expected = if row["expected_error"] == "Unsupported" {
                ErrorCode::Unsupported
            } else {
                ErrorCode::InvalidValue
            };
            assert_eq!(result.unwrap_err().code, expected);
            continue;
        }
        assert_eq!(
            result.is_ok(),
            tokens[0] == "0",
            "{}: {result:?}",
            row["id"]
        );
        if let Ok(sdf) = result {
            let joint = &sdf.models[0].joints[0];
            let mut index = 1;
            for axis in [joint.axis.as_ref(), joint.axis2.as_ref()] {
                assert_eq!(axis.is_some(), tokens[index] == "1");
                index += 1;
                if let Some(axis) = axis {
                    assert_eq!(axis.dynamics.source.is_some(), tokens[index] == "1");
                    index += 1;
                    let values = [
                        axis.dynamics.damping,
                        axis.dynamics.friction,
                        axis.dynamics.spring_reference,
                        axis.dynamics.spring_stiffness,
                        axis.stiffness,
                        axis.dissipation,
                    ];
                    for value in values {
                        assert_eq!(
                            value.to_bits(),
                            tokens[index].as_str().unwrap().parse::<u64>().unwrap(),
                            "{}",
                            row["id"]
                        );
                        index += 1;
                    }
                }
            }
            assert_eq!(index, tokens.len());
        }
    }
}

#[test]
fn axis_dynamics_owns_declaration_without_creating_runtime_effects() {
    let dynamics = {
        let sdf = load_str("<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child><axis><dynamics><damping>0.1</damping><friction>-2</friction></dynamics></axis></joint></model></sdf>", "cached/joint.sdf", &NoResources).unwrap();
        let axis = sdf.models[0].joints[0].axis.as_ref().unwrap();
        assert_eq!(axis.stiffness, 1e8);
        assert_eq!(axis.dissipation, 1.0);
        axis.dynamics.clone()
    };
    assert_eq!(dynamics.damping, 0.1);
    assert_eq!(dynamics.friction, -2.0);
    assert_eq!(dynamics.spring_reference, 0.0);
    assert_eq!(dynamics.spring_stiffness, 0.0);
    assert_eq!(dynamics.source.unwrap().uri, "cached/joint.sdf");
    assert_eq!(
        dynamics.xml.unwrap(),
        "<dynamics><damping>0.1</damping><friction>-2</friction></dynamics>"
    );
}
