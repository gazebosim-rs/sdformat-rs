// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::World;
use serde_json::Value;

pub fn compare(world: &World, native: &Value) {
    let lines: Vec<_> = native["stdout"].as_str().unwrap().lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "ROOT");
    let tokens: Vec<_> = lines[1].split_whitespace().collect();
    assert_eq!(tokens.len(), 10);
    assert_eq!(tokens[0], "VALUES");
    for (actual, token) in world
        .wind_linear_velocity
        .into_iter()
        .chain(world.magnetic_field)
        .chain(world.gravity)
        .zip(&tokens[1..])
    {
        assert_eq!(actual.to_bits(), token.parse::<u64>().unwrap());
    }
}
