// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, load_str};
use serde_json::Value;

fn string(token: &Value) -> String {
    let value = token.as_str().unwrap();
    if value == "-" {
        return String::new();
    }
    let bytes = value
        .as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    String::from_utf8(bytes).unwrap()
}

#[test]
fn native_physics_profiles_and_default_selection() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("physics.json")).unwrap();
    for row in rows {
        let result = load_str(row["xml"].as_str().unwrap(), "world.sdf", &NoResources);
        let tokens = row["tokens"].as_array().unwrap();
        if row.get("gap").is_some() {
            assert_eq!(tokens[0], "0", "{}", row["id"]);
            let expected = if row["gap"].as_str().unwrap().starts_with("Existing") {
                ErrorCode::InvalidValue
            } else {
                ErrorCode::Unsupported
            };
            assert_eq!(result.unwrap_err().code, expected, "{}", row["id"]);
            continue;
        }
        assert_eq!(
            result.is_ok(),
            tokens[0] == "0",
            "{}: {result:?}",
            row["id"]
        );
        if let Ok(sdf) = result {
            let world = &sdf.worlds[0];
            assert_eq!(
                world.physics.len(),
                tokens[1].as_str().unwrap().parse::<usize>().unwrap()
            );
            let chosen = tokens[2].as_str().unwrap().parse::<usize>().unwrap();
            assert!(std::ptr::eq(
                world.physics_default().unwrap(),
                &world.physics[chosen]
            ));
            for (index, profile) in world.physics.iter().enumerate() {
                let t = &tokens[3 + index * 8..3 + (index + 1) * 8];
                assert_eq!(profile.name, string(&t[0]), "{}", row["id"]);
                assert_eq!(profile.is_default, t[1] == "1");
                assert_eq!(profile.engine_type, string(&t[2]));
                assert_eq!(
                    profile.max_step_size.to_bits(),
                    t[3].as_str().unwrap().parse::<u64>().unwrap()
                );
                assert_eq!(
                    profile.real_time_factor.to_bits(),
                    t[4].as_str().unwrap().parse::<u64>().unwrap()
                );
                assert_eq!(
                    profile.max_contacts,
                    t[5].as_str().unwrap().parse::<i32>().unwrap()
                );
                assert_eq!(
                    profile.real_time_update_rate.to_bits(),
                    t[7].as_str().unwrap().parse::<u64>().unwrap()
                );
            }
        }
    }
}

#[test]
fn physics_profiles_own_provenance_and_explicitness() {
    let mut sdf = load_str("<sdf version='1.12'><world name='w'><physics name='p' type='custom'><max_step_size>0.01</max_step_size></physics></world></sdf>", "cached/world.sdf", &NoResources).unwrap();
    let profile = sdf.worlds[0].physics_default().unwrap().clone();
    assert_eq!(profile.source.as_ref().unwrap().uri, "cached/world.sdf");
    assert_eq!(
        profile.xml.as_ref().unwrap(),
        "<physics name='p' type='custom'><max_step_size>0.01</max_step_size></physics>"
    );
    sdf.worlds[0].physics.clear();
    assert!(sdf.worlds[0].physics_default().is_none());
    drop(sdf);
    assert_eq!(profile.max_step_size, 0.01);
    let sdf = load_str(
        "<sdf version='1.12'><world name='w'/></sdf>",
        "implicit",
        &NoResources,
    )
    .unwrap();
    let implicit = sdf.worlds[0].physics_default().unwrap();
    assert_eq!(implicit.name, "default_physics");
    assert!(implicit.source.is_none() && implicit.xml.is_none());
}
