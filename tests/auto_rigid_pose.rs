// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "common/auto_cylinder_inertial.rs"]
mod cylinder;
#[path = "common/auto_rigid_pose.rs"]
mod rigid;
use sdformat::*;
use serde_json::Value;

fn cases() -> Vec<Value> {
    let value: Value = serde_json::from_str(include_str!("auto_rigid_pose_cases.json")).unwrap();
    value["cases"].as_array().unwrap().clone()
}

fn old_case(sphere: bool) -> Value {
    cases()
        .into_iter()
        .find(|row| {
            row["name"]
                == if sphere {
                    "sphere_old_rigid"
                } else {
                    "box_old_rigid"
                }
        })
        .unwrap()
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
fn actual_complete_native_combined_corpus_preserves_existing_numeric_policy() {
    let cases = cases();
    let native: Value = serde_json::from_str(include_str!("auto_rigid_pose_native.json")).unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    assert_eq!(
        rows.len(),
        cases.len(),
        "All declared actual native observations are required; empty data cannot qualify"
    );
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
        if case["name"] == "box_old_rigid" || case["name"] == "sphere_old_rigid" {
            rigid::assert_translated_rotation(
                case["xml"].as_str().unwrap(),
                case["uri"].as_str().unwrap(),
                case["name"] == "sphere_old_rigid",
            );
        }
        assert_eq!(row["root_values"].as_array().unwrap().len(), 2);
        assert_eq!(row["root_status"].as_array().unwrap().len(), 2);
        assert_eq!(row["collision_moi"].as_array().unwrap().len(), 9);
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
            rigid::four_ulps(actual, direct[index].as_u64().unwrap());
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
            rigid::four_ulps(actual, *expected);
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
fn unsupported_encodings_overrides_geometry_and_multiplicity_stay_closed() {
    for sphere in [false, true] {
        let row = old_case(sphere);
        let original = row["xml"].as_str().unwrap();
        for attribute in [
            "relative_to=''",
            "relative_to='link'",
            "degrees='false'",
            "rotation_format='euler_rpy'",
        ] {
            let xml = original.replace("<pose>", &format!("<pose {attribute}>"));
            let error = load_str(&xml, "closed-combined.sdf", &NoResources).unwrap_err();
            assert_eq!(error.code, ErrorCode::Unsupported);
            assert_eq!(error.source.uri, "closed-combined.sdf");
        }
        let quaternion = original.replace(
            "<pose>1 0 0 0.1 0 0</pose>",
            "<pose rotation_format='quat_xyzw'>1 0 0 0 0 0 1</pose>",
        );
        assert_eq!(
            load_str(&quaternion, "quaternion.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        let density = if sphere {
            "<density>2170</density>"
        } else {
            "<density>1240</density>"
        };
        for invalid in ["0", "-1", "NaN", "inf"] {
            let xml = original.replace(density, &format!("<density>{invalid}</density>"));
            assert_eq!(
                load_str(&xml, "density.sdf", &NoResources)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidValue
            );
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
        let many = original.replace("</link>", "<collision name='second'/></link>");
        assert_eq!(
            load_str(&many, "multiple.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        let none = "<sdf version='1.12'><model name='m'><link name='l'><inertial auto='true'/></link></model></sdf>";
        assert_eq!(
            load_str(none, "no-collision.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let cylinder = "<sdf version='1.12'><model name='m'><link name='l'><inertial auto='true'/><collision name='c'><pose>1 2 3 0.1 0.2 0.3</pose><geometry><cylinder><radius>1</radius><length>2</length></cylinder></geometry></collision></link></model></sdf>";
    cylinder::assert_retained_input(cylinder, "cylinder.sdf");
}

#[test]
fn imported_combined_pose_retains_owned_source_and_local_center() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!((uri, parent), ("rigid", "world.sdf"));
            Ok(Resource {
                canonical_uri: "canonical://rigid.sdf".into(),
                xml: old_case(false)["xml"].as_str().unwrap().into(),
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>rigid</uri><name>imported</name><pose>4 5 6 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    let link = &model.links[0];
    assert_eq!(link.inertial.pose.transform.translation, [1., 0., 0.]);
    assert_eq!(
        sdf.resolve_pose(
            &link.inertial.pose,
            "w::imported",
            "w::imported::link",
            "::"
        )
        .unwrap()
        .translation,
        [5., 5., 6.]
    );
    for source in [&model.source, &link.source, &link.collisions[0].source] {
        assert_eq!(source.uri, "canonical://rigid.sdf");
        assert!(source.line > 0 && source.column > 0);
    }
    let mut copy = model.clone();
    assert_eq!(copy.element(), model.element());
    copy.links[0].collisions[0].density = Some(5.);
    copy.links[0].collisions[0].pose.transform.translation = [8., 0., 0.];
    copy.links[0].inertial.pose.transform.translation = [9., 0., 0.];
    assert_eq!(link.collisions[0].density, Some(1240.));
    assert_eq!(link.collisions[0].pose.transform.translation, [1., 0., 0.]);
    assert_eq!(link.inertial.pose.transform.translation, [1., 0., 0.]);
    sdf.validate_non_frame_poses().unwrap();
}
