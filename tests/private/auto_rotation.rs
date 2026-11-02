// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use super::automatic_rotated_properties;

#[test]
fn selected_native_raw_properties_precede_rotation_for_all_captured_inputs() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../auto_rotation_pose_cases.json")).unwrap();
    let native: serde_json::Value =
        serde_json::from_str(include_str!("../auto_rotation_pose_native.json")).unwrap();
    let cases = cases["cases"].as_array().unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert!(!rows.is_empty());
    assert_eq!(rows.len(), cases.len());
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
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
        for (index, actual) in std::iter::once(mass).chain(principal).enumerate() {
            assert_eq!(
                actual.to_bits(),
                expected[index].as_u64().unwrap(),
                "{} scalar{index}",
                case["name"]
            );
        }
    }
    assert!(automatic_rotated_properties(&crate::Geometry::Empty, 1.).is_none());
}

#[test]
fn sdk_intermediate_overflow_is_refused_after_the_existing_kernel_boundary() {
    use parry3d_f64::{mass_properties::MassProperties, math::Vector};
    for (size, density) in [([2e154, 1., 1.], 1e-154), ([2e154, 2e154, 1e-154], 1e-155)] {
        let properties = MassProperties::from_cuboid(
            density,
            Vector::new(size[0] / 2., size[1] / 2., size[2] / 2.),
        );
        assert!(properties.mass().is_finite() && properties.mass() > 0.);
        assert!(
            properties
                .principal_inertia()
                .to_array()
                .iter()
                .all(|v| v.is_finite() && *v > 0.)
        );
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><inertial auto='true'/><collision name='c'><pose>0 0 0 0.1 0 0</pose><density>{density}</density><geometry><box><size>{} {} {}</size></box></geometry></collision></link></model></sdf>",
            size[0], size[1], size[2]
        );
        let error = crate::load_str(&xml, "sdk-overflow.sdf", &crate::NoResources).unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::Unsupported);
        assert_eq!(
            error.message,
            "rotated primitive properties exceed the finite numeric domain"
        );
        assert_eq!(error.source.uri, "sdk-overflow.sdf");
    }
}
