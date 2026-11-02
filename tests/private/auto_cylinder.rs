// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use super::automatic_rotated_properties;

#[test]
fn cylinder_scalar_order_matches_all_valid_actual_direct_values() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../auto_cylinder_inertial_cases.json")).unwrap();
    let native: serde_json::Value =
        serde_json::from_str(include_str!("../auto_cylinder_inertial_native.json")).unwrap();
    let cases = cases["cases"].as_array().unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    assert_eq!(rows.len(), cases.len());
    let mut compared = 0;
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
        if row["collision_status"] != serde_json::json!([1, 1])
            || !row["collision_errors"].as_array().unwrap().is_empty()
        {
            continue;
        }
        let xml = case["xml"]
            .as_str()
            .unwrap()
            .replace("auto='true'", "auto='false'");
        let parsed =
            crate::load_str(&xml, case["uri"].as_str().unwrap(), &crate::NoResources).unwrap();
        let collision = &parsed.models[0].links[0].collisions[0];
        let (mass, principal) =
            automatic_rotated_properties(&collision.geometry, collision.density()).unwrap();
        let expected = row["collision_values"].as_array().unwrap();
        assert_eq!(expected.len(), 14);
        for (actual, value) in std::iter::once(mass).chain(principal).zip(expected) {
            assert_eq!(
                actual.to_bits(),
                value.as_u64().unwrap(),
                "{}",
                case["name"]
            );
        }
        compared += 1;
    }
    assert!(
        compared > 0,
        "Actual valid direct Cylinder values are required"
    );
}
