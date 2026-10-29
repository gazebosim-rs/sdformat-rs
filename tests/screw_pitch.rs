// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{NoResources, load_str};
use serde_json::Value;

#[test]
fn native_signed_zero_extremes_and_unit_conversion_bits_match() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("screw_pitch.json")).unwrap();
    for row in rows {
        let sdf = load_str(row["xml"].as_str().unwrap(), "pitch.sdf", &NoResources).unwrap();
        let joint = &sdf.models[0].joints[0];
        let tokens = row["tokens"].as_array().unwrap();
        assert_eq!(tokens[0], "0");
        for (actual, expected) in [joint.screw_thread_pitch, joint.legacy_thread_pitch()]
            .into_iter()
            .zip(&tokens[1..])
        {
            let expected: u64 = expected.as_str().unwrap().parse().unwrap();
            assert_eq!(actual.to_bits(), expected, "{}", row["id"]);
        }
    }
}
#[test]
fn classic_pitch_tracks_owned_edits_without_mutating_copies() {
    let sdf = load_str(
        include_str!("fixtures/joint_screw_thread_pitch.sdf"),
        "pitch.sdf",
        &NoResources,
    )
    .unwrap();
    let original = &sdf.models[0].joints[0];
    let mut edited = original.clone();
    edited.screw_thread_pitch = -0.5;
    assert_eq!(edited.legacy_thread_pitch(), 2. * std::f64::consts::TAU);
    assert_eq!(original.legacy_thread_pitch(), -std::f64::consts::TAU);
}
