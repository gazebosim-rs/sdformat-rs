// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "common/auto_capsule_translation.rs"]
mod translation;
use sdformat::*;
use serde_json::Value;

fn cases() -> Vec<Value> {
    let value: Value =
        serde_json::from_str(include_str!("auto_capsule_translation_cases.json")).unwrap();
    value["cases"].as_array().unwrap().clone()
}

fn selected(name: &str) -> Value {
    cases().into_iter().find(|row| row["name"] == name).unwrap()
}

fn manual_policy(values: &[u64]) -> Result<Sdf, Diagnostic> {
    let value = |i| f64::from_bits(values[i]).to_string();
    let xml = format!(
        "<sdf version='1.12'><model name='policy'><link name='body'><inertial><mass>{}</mass><inertia><ixx>{}</ixx><iyy>{}</iyy><izz>{}</izz><ixy>{}</ixy><ixz>{}</ixz><iyz>{}</iyz></inertia></inertial></link></model></sdf>",
        value(2),
        value(3),
        value(4),
        value(5),
        value(6),
        value(7),
        value(8)
    );
    load_str(&xml, "existing-manual-policy.sdf", &NoResources)
}

#[test]
fn actual_capsule_translations_preserve_native_aggregate_and_numeric_policy() {
    let cases = cases();
    let native: Value =
        serde_json::from_str(include_str!("auto_capsule_translation_native.json")).unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    assert_eq!(
        rows.len(),
        cases.len(),
        "All declared actual native observations are required; empty data cannot qualify"
    );
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
        if let Some(name @ ("old_translated_corpus" | "old_translated_closed")) =
            case["name"].as_str()
        {
            translation::assert_translated_input(
                case["xml"].as_str().unwrap(),
                case["uri"].as_str().unwrap(),
                name,
            );
        }
        assert_eq!(row["root_values"].as_array().unwrap().len(), 2);
        assert_eq!(row["root_status"].as_array().unwrap().len(), 2);
        assert_eq!(row["collision_moi"].as_array().unwrap().len(), 9);
        let native_errors = if matches!(
            case["name"].as_str().unwrap(),
            "maximum_density" | "native_density_prefix_overflow"
        ) {
            serde_json::json!([19])
        } else {
            serde_json::json!([])
        };
        assert_eq!(row["load_errors"], native_errors);
        assert_eq!(row["collision_errors"], native_errors);
        assert!(row["resolve_errors"].as_array().unwrap().is_empty());
        let overflow = matches!(
            case["name"].as_str().unwrap(),
            "overflow_translation" | "maximum_translation"
        );
        assert_eq!(
            row["root_status"],
            if overflow {
                serde_json::json!([[0, 0], [0, 0]])
            } else {
                serde_json::json!([[1, 1], [1, 1]])
            }
        );
        assert_eq!(row["collision_status"], serde_json::json!([1, 1]));
        assert_eq!(row["root_values"][0], row["root_values"][1]);
        let warns = matches!(
            case["name"].as_str().unwrap(),
            "old_translated_corpus"
                | "negative_radius"
                | "zero_length"
                | "both_nonpositive"
                | "missing_density"
        );
        assert_eq!(!row["stderr"].as_str().unwrap().is_empty(), warns);
        if case["name"] == "integer_translation" {
            assert_ne!(
                row["root_values"][1][11], row["collision_values"][9],
                "Actual native center has a nonzero-component residual"
            );
        }
        let result = load_str(
            case["xml"].as_str().unwrap(),
            case["uri"].as_str().unwrap(),
            &NoResources,
        );
        if case["rust_error"] == "Unsupported"
            || row["root_status"][1][0] == 0
            || row["root_status"][1][1] == 0
        {
            assert_eq!(
                result.unwrap_err().code,
                ErrorCode::Unsupported,
                "{}",
                case["name"]
            );
            continue;
        }
        assert!(row["load_errors"].as_array().unwrap().is_empty());
        assert!(row["resolve_errors"].as_array().unwrap().is_empty());
        assert!(row["collision_errors"].as_array().unwrap().is_empty());
        assert_eq!(row["root_status"], serde_json::json!([[1, 1], [1, 1]]));
        assert_eq!(row["collision_status"], serde_json::json!([1, 1]));
        assert_eq!(row["root_values"][0], row["root_values"][1]);
        let values: Vec<u64> = row["root_values"][1]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        assert_eq!(values.len(), 18);
        assert_eq!(&values[..2], &[1, 1]);
        assert_eq!(&values[16..], &[0, 0]);
        if let Err(error) = manual_policy(&values) {
            assert_eq!(error.code, ErrorCode::InvalidValue);
            assert_eq!(result.unwrap_err().code, ErrorCode::InvalidValue);
            continue;
        }
        let pose_tokens: Vec<_> = case["pose_text"]
            .as_str()
            .unwrap()
            .split_whitespace()
            .collect();
        assert_eq!(pose_tokens.len(), 6);
        let origin = format!(
            "<pose>0 0 0 {} {} {}</pose>",
            pose_tokens[3], pose_tokens[4], pose_tokens[5]
        );
        let origin_xml = case["xml"].as_str().unwrap().replace(
            &format!("<pose>{}</pose>", case["pose_text"].as_str().unwrap()),
            &origin,
        );
        let origin = load_str(&origin_xml, "direct-rotation-control.sdf", &NoResources).unwrap();
        let direct = row["collision_moi"].as_array().unwrap();
        for (actual, index) in origin.models[0].links[0]
            .inertial
            .inertia
            .into_iter()
            .zip([0, 4, 8, 1, 2, 5])
        {
            translation::four_ulps(actual, direct[index].as_u64().unwrap());
        }
        let sdf = result.unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
        let link = &sdf.models[0].links[0];
        let inertia = &link.inertial;
        for (actual, expected) in std::iter::once(inertia.mass)
            .chain(inertia.inertia)
            .chain(inertia.pose.transform.translation)
            .chain(inertia.pose.transform.rotation)
            .zip(&values[2..16])
        {
            translation::four_ulps(actual, *expected);
        }
        assert!(inertia.pose.relative_to.is_empty());
        assert!(inertia.fluid_added_mass.is_none());
        let manual = load_str(
            &case["xml"]
                .as_str()
                .unwrap()
                .replace("auto='true'", "auto='false'"),
            case["uri"].as_str().unwrap(),
            &NoResources,
        )
        .unwrap();
        let collision = &link.collisions[0];
        let old_collision = &manual.models[0].links[0].collisions[0];
        assert_eq!(collision.name, old_collision.name);
        assert_eq!(collision.pose, old_collision.pose);
        assert_eq!(collision.geometry, old_collision.geometry);
        assert_eq!(collision.surface, old_collision.surface);
        assert_eq!(collision.density, old_collision.density);
        assert_eq!(collision.source.uri, case["uri"].as_str().unwrap());
        assert_eq!(collision.density(), old_collision.density());
        let default = Inertial::default();
        let old = &manual.models[0].links[0].inertial;
        assert_eq!(old.mass, default.mass);
        assert_eq!(old.inertia, default.inertia);
        assert_eq!(old.pose, default.pose);
        assert_eq!(old.fluid_added_mass, default.fluid_added_mass);
        sdf.validate_non_frame_poses().unwrap();
    }
}

#[test]
fn capsule_translation_retains_rotation_attribute_override_and_multiplicity_refusals() {
    let row = selected("old_translated_closed");
    let original = row["xml"].as_str().unwrap();
    for pose in [
        "<pose>0 0 0 0.1 0 0</pose>",
        "<pose>1 0 0 0.1 0 0</pose>",
        "<pose relative_to=''>1 0 0 0 0 0</pose>",
        "<pose relative_to='l'>1 0 0 0 0 0</pose>",
        "<pose degrees='false'>1 0 0 0 0 0</pose>",
        "<pose rotation_format='euler_rpy'>1 0 0 0 0 0</pose>",
        "<pose rotation_format='quat_xyzw'>1 0 0 0 0 0 1</pose>",
    ] {
        let xml = original.replace("<pose>1 0 0 0 0 0</pose>", pose);
        let error = load_str(&xml, "closed-translation.sdf", &NoResources).unwrap_err();
        assert_eq!(error.code, ErrorCode::Unsupported);
        assert_eq!(error.source.uri, "closed-translation.sdf");
    }
    for override_xml in [
        "<mass>1</mass>",
        "<pose/>",
        "<inertia/>",
        "<density>1</density>",
    ] {
        let xml = original.replace(
            "<inertial auto='true'/>",
            &format!("<inertial auto='true'>{override_xml}</inertial>"),
        );
        assert_eq!(
            load_str(&xml, "override.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    for density in ["0", "-1", "NaN", "inf"] {
        let xml = original.replace(
            "<density>2710</density>",
            &format!("<density>{density}</density>"),
        );
        assert_eq!(
            load_str(&xml, "density.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::InvalidValue
        );
    }
    let multiple = original.replace(
        "</link>",
        "<collision name='second'><geometry><capsule/></geometry></collision></link>",
    );
    assert_eq!(
        load_str(&multiple, "multiple.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}

#[test]
fn translated_capsule_import_clone_and_frames_keep_canonical_source_and_local_center() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!((uri, parent), ("capsule", "world.sdf"));
            Ok(Resource {
                canonical_uri: "canonical://translated-capsule.sdf".into(),
                xml: selected("old_translated_corpus")["xml"]
                    .as_str()
                    .unwrap()
                    .into(),
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>capsule</uri><name>imported</name><pose>4 5 6 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    let link = &model.links[0];
    let input = selected("old_translated_corpus");
    let direct = load_str(
        input["xml"].as_str().unwrap(),
        "direct-control.sdf",
        &NoResources,
    )
    .unwrap();
    let center = direct.models[0].links[0]
        .inertial
        .pose
        .transform
        .translation;
    assert_eq!(link.inertial.pose.transform.translation, center);
    assert_eq!(
        sdf.resolve_pose(&link.inertial.pose, "w::imported", "w::imported::l", "::")
            .unwrap()
            .translation,
        [center[0] + 4., center[1] + 5., center[2] + 6.]
    );
    for source in [&model.source, &link.source, &link.collisions[0].source] {
        assert_eq!(source.uri, "canonical://translated-capsule.sdf");
        assert!(source.line > 0 && source.column > 0);
    }
    let mut copy = model.clone();
    assert_eq!(copy.element(), model.element());
    copy.links[0].collisions[0].density = Some(5.);
    copy.links[0].collisions[0].pose.transform.translation = [8., 0., 0.];
    copy.links[0].inertial.pose.transform.translation = [9., 0., 0.];
    assert_eq!(link.collisions[0].density, None);
    assert_eq!(link.collisions[0].pose.transform.translation, [1., 2., 3.]);
    assert_eq!(link.inertial.pose.transform.translation, center);
    sdf.validate_non_frame_poses().unwrap();
}
