// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{NoResources, load_str};

#[test]
fn first_native_positive_axis_above_float_epsilon_is_normalized() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("axis_limits.json")).unwrap();
    let row = cases
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "axis_xyz_1e_minus_6")
        .unwrap();
    assert_eq!(row["tokens"][0], "0");
    let sdf = load_str(
        row["xml"].as_str().unwrap(),
        "axis-boundary.sdf",
        &NoResources,
    )
    .unwrap();
    assert_eq!(
        sdf.models[0].joints[0].axis.as_ref().unwrap().xyz,
        [1., 0., 0.]
    );
}
