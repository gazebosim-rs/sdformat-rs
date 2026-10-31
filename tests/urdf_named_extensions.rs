// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use serde_json::{Value, json};
fn projection(import: &sdformat::UrdfImport) -> Value {
    let link = &import.sdf.models[0].links[0];
    let visual = &link.visuals[0];
    let collision = &link.collisions[0];
    let script = visual
        .material
        .as_ref()
        .and_then(|material| material.script.as_ref());
    json!({"visual_name": visual.name, "collision_name": collision.name,
        "material_name": script.map(|s| &s.name), "material_uri": script.map(|s| &s.uris[0]),
        "mu1_bits": collision.surface.friction.ode.mu.to_bits(), "mu2_bits": collision.surface.friction.ode.mu2.to_bits()})
}
fn data() -> Value {
    serde_json::from_str(include_str!("urdf_named_extensions.json")).unwrap()
}
#[test]
fn native_named_extension_targets_are_applied_or_explicitly_rejected() {
    for row in data()["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        let result = sdformat::load_urdf_str(xml, "named.urdf");
        if row["rust_policy"] == "accepted" {
            let import = result.unwrap_or_else(|e| panic!("{name}: {e:?}"));
            assert_eq!(projection(&import), row["native_projection"], "{name}");
            assert_eq!(import.original_xml(), xml);
            import.sdf.validate_non_frame_poses().unwrap();
        } else {
            let error = result.expect_err(name);
            assert_eq!(error.code, sdformat::ErrorCode::Unsupported, "{name}");
            assert_eq!(error.source.uri, "named.urdf");
            let at = xml.find("<gazebo").unwrap();
            assert_eq!(
                error.source.line as usize,
                xml[..at].bytes().filter(|c| *c == b'\n').count() + 1,
                "{name}"
            );
            assert_eq!(
                error.source.column as usize,
                xml[..at].rsplit('\n').next().unwrap().chars().count() + 1,
                "{name}"
            );
            assert_eq!(
                error.message,
                "Gazebo metadata cannot target a native lump-prefixed view name"
            );
        }
    }
}
#[test]
fn observed_accepted_release_mismatch_is_material_and_friction_not_view_naming() {
    let data = data();
    assert_eq!(
        data["accepted_revision"],
        "74fa759fd11c019c72eac1b596e62273e77478fd"
    );
    for name in ["outside_visual", "outside_collision"] {
        let row = data["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == name)
            .unwrap();
        let old = &row["accepted_74fa759_projection"];
        let native = &row["native_projection"];
        assert_eq!(old["visual_name"], native["visual_name"]);
        assert_eq!(old["collision_name"], native["collision_name"]);
        if name == "outside_visual" {
            assert_eq!(old["material_name"], "Gazebo/Orange");
            assert!(native["material_name"].is_null());
            assert_eq!(old["mu1_bits"], native["mu1_bits"]);
            assert_eq!(old["mu2_bits"], native["mu2_bits"]);
        } else {
            assert_eq!(old["material_name"], native["material_name"]);
            assert_eq!(old["mu1_bits"].as_u64().unwrap(), 100_f64.to_bits());
            assert_eq!(old["mu2_bits"].as_u64().unwrap(), 1000_f64.to_bits());
            assert_eq!(native["mu1_bits"].as_u64().unwrap(), 1_f64.to_bits());
            assert_eq!(native["mu2_bits"].as_u64().unwrap(), 1_f64.to_bits());
        }
    }
}
#[test]
fn all_prior_accepted_single_link_generated_xml_bytes_remain_identical() {
    let corpus: Value = serde_json::from_str(include_str!("urdf_single_link.json")).unwrap();
    let expected: Value =
        serde_json::from_str(include_str!("urdf_single_link_converted.json")).unwrap();
    let mut count = 0;
    for row in corpus["cases"].as_array().unwrap() {
        if row["rust_policy"] == "accepted" {
            let name = row["name"].as_str().unwrap();
            let import =
                sdformat::load_urdf_str(row["xml"].as_str().unwrap(), "input.urdf").unwrap();
            assert_eq!(
                import.converted_xml(),
                expected[name].as_str().unwrap(),
                "{name}"
            );
            count += 1;
        }
    }
    assert_eq!(count, 31);
    assert_eq!(count, expected.as_object().unwrap().len());
}
