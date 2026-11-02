// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "common/auto_mass_override.rs"]
mod mass_override;
use sdformat::*;
use serde_json::Value;

#[test]
fn actual_origin_mass_overrides_preserve_native_values_or_explicit_numeric_refusals() {
    let cases = mass_override::cases();
    let native: Value =
        serde_json::from_str(include_str!("auto_mass_override_native.json")).unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    assert_eq!(
        rows.len(),
        cases.len(),
        "Empty or incomplete actual observations cannot qualify"
    );
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
        let xml = case["xml"].as_str().unwrap();
        let uri = case["uri"].as_str().unwrap();
        if case["name"] == "minimum_subnormal_mass" {
            assert_eq!(row["native_load_refusal"], true);
            assert_eq!(row["exit_code"], 2);
            assert_eq!(row["maps_recorded"], false);
            assert_eq!(row["load_errors"], serde_json::json!([9, 9, 9, 9, 9, 30]));
            assert_eq!(row["stdout"].as_str().unwrap(), "LOAD 9 9 9 9 9 30\n");
            assert_eq!(
                row["stderr"].as_str().unwrap(),
                "\u{1b}[1;31mError [Param.cc:1288]\u{1b}[0m Error Code 34: Msg: Out of range. Unable to set value [5e-324 ] for key[mass].\n"
            );
            for field in [
                "root_values",
                "root_status",
                "collision_values",
                "collision_status",
                "collision_moi",
                "collision_errors",
                "resolve_errors",
                "mass_specified",
                "direct_warn_default",
            ] {
                assert!(
                    row.get(field).is_none(),
                    "unavailable native state must not be fabricated"
                );
            }
            let error = load_str(xml, uri, &NoResources).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidValue);
            assert_eq!(error.source.uri, uri);
            continue;
        }
        assert_eq!(row["exit_code"], 0);
        assert_eq!(row["maps_recorded"], true);
        let errors = if case["name"] == "initial_density_overflow" {
            serde_json::json!([19])
        } else {
            serde_json::json!([])
        };
        assert_eq!(row["load_errors"], errors);
        assert_eq!(row["collision_errors"], errors);
        assert_eq!(row["resolve_errors"], serde_json::json!([]));
        let flags = match case["name"].as_str().unwrap() {
            "negative_mass" => serde_json::json!([[1, 0], [1, 0]]),
            "nonfinite_nan" | "nonfinite_inf" | "maximum_mass" | "initial_density_overflow" => {
                serde_json::json!([[0, 0], [0, 0]])
            }
            _ => serde_json::json!([[1, 1], [1, 1]]),
        };
        assert_eq!(row["root_status"], flags);
        assert_eq!(row["collision_status"], serde_json::json!([1, 1]));
        assert_eq!(row["stderr"].as_str().unwrap(), "");
        if case["name"] == "zero_mass" {
            assert!(
                row["root_values"][1].as_array().unwrap()[2..9]
                    .iter()
                    .all(|value| f64::from_bits(value.as_u64().unwrap()) == 0.)
            );
        }
        assert_eq!(row["root_values"].as_array().unwrap().len(), 2);
        assert_eq!(row["root_status"].as_array().unwrap().len(), 2);
        assert_eq!(row["collision_values"].as_array().unwrap().len(), 14);
        assert_eq!(row["collision_moi"].as_array().unwrap().len(), 9);
        assert_eq!(row["mass_specified"], 1);
        assert_eq!(row["direct_warn_default"], 0);
        assert_eq!(row["root_values"][0], row["root_values"][1]);
        if let Some(expected) = case["rust_error"].as_str() {
            let error = load_str(xml, uri, &NoResources).unwrap_err();
            assert_eq!(
                error.code,
                if expected == "Unsupported" {
                    ErrorCode::Unsupported
                } else {
                    assert_eq!(expected, "InvalidValue");
                    ErrorCode::InvalidValue
                }
            );
            assert_eq!(error.source.uri, uri);
        } else {
            mass_override::assert_mass_override(xml, uri, case["name"].as_str().unwrap());
        }
    }
}

#[test]
fn mass_override_keeps_other_pose_geometry_override_and_density_domains_closed() {
    let case = mass_override::cases()
        .into_iter()
        .find(|row| row["name"] == "unit_box_four")
        .unwrap();
    let xml = case["xml"].as_str().unwrap();
    for pose in [
        "<pose>1 0 0 0 0 0</pose>",
        "<pose>0 0 0 0.1 0 0</pose>",
        "<pose relative_to=''>0 0 0 0 0 0</pose>",
        "<pose relative_to='l'>0 0 0 0 0 0</pose>",
        "<pose degrees='true'>0 0 0 0 0 0</pose>",
        "<pose rotation_format='quat_xyzw'>0 0 0 0 0 0 1</pose>",
    ] {
        let input = xml.replace("<geometry>", &format!("{pose}<geometry>"));
        assert_eq!(
            load_str(&input, "closed-mass.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    for extra in [
        "<pose/>",
        "<inertia/>",
        "<density>1</density>",
        "<fluid_added_mass/>",
    ] {
        let input = xml.replace("</inertial>", &format!("{extra}</inertial>"));
        assert_eq!(
            load_str(&input, "extra-inertial.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    for shape in [
        "<cylinder><radius>1</radius><length>2</length></cylinder>",
        "<capsule><radius>1</radius><length>2</length></capsule>",
    ] {
        let input = xml.replace("<box><size>1 1 1</size></box>", shape);
        assert_eq!(
            load_str(&input, "other-shape.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    for density in ["0", "-1", "nan", "inf"] {
        let input = xml.replace(
            "<density>2</density>",
            &format!("<density>{density}</density>"),
        );
        assert_eq!(
            load_str(&input, "invalid-density.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::InvalidValue
        );
    }
    let multiple = xml.replace("</link>", "<collision name='second'><geometry><sphere><radius>1</radius></sphere></geometry></collision></link>");
    assert_eq!(
        load_str(&multiple, "multiple.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    for mass in [
        "0",
        "-0",
        "-1",
        "nan",
        "inf",
        "-inf",
        "1e400",
        "5e-324",
        "not-a-number",
    ] {
        let input = xml.replace("<mass>4</mass>", &format!("<mass>{mass}</mass>"));
        assert_eq!(
            load_str(&input, "invalid-mass.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::InvalidValue
        );
    }
    let attributed = xml.replace("<mass>4</mass>", "<mass unknown='x'>4</mass>");
    assert_eq!(
        load_str(&attributed, "mass-attribute.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let empty = "<sdf version='1.12'><model name='m'><link name='l'><inertial auto='true'><mass>2</mass></inertial></link></model></sdf>";
    assert_eq!(
        load_str(empty, "no-collision.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}

#[test]
fn mass_override_preserves_density_presence_and_equivalent_origin_pose_values() {
    for shape in ["box_old_identity", "sphere_old_identity"] {
        let row = mass_override::cases()
            .into_iter()
            .find(|row| row["name"] == shape)
            .unwrap();
        let xml = row["xml"].as_str().unwrap();
        let start = xml.find("<density>").unwrap();
        let end = xml[start..].find("</density>").unwrap() + start + "</density>".len();
        let base = format!("{}{}", &xml[..start], &xml[end..]);
        for (density, presence) in [
            ("", None),
            ("<density/>", Some(1000.)),
            ("<density> </density>", Some(1000.)),
            ("<density>1000</density>", Some(1000.)),
        ] {
            let input = base.replace("<geometry>", &format!("{density}<geometry>"));
            for pose in [
                "<pose/>",
                "<pose>0.0 +0.00 0e0 0E0 0.0 0</pose>",
                "<pose degrees='0' rotation_format='euler_rpy'>0 0 0 0 0 0</pose>",
            ] {
                let input = input.replace("<pose>0 0 0 0 0 0</pose>", pose);
                let sdf = load_str(&input, "density-pose.sdf", &NoResources).unwrap();
                let link = &sdf.models[0].links[0];
                assert_eq!(link.inertial.mass, 1.);
                assert_eq!(link.inertial.pose, Pose::default());
                assert_eq!(link.collisions[0].pose, Pose::default());
                assert_eq!(link.collisions[0].density, presence);
                assert_eq!(link.collisions[0].density(), 1000.);
                assert_eq!(link.source.uri, "density-pose.sdf");
                sdf.validate_non_frame_poses().unwrap();
            }
        }
    }
}

#[test]
fn mass_override_canonical_include_and_clone_keep_local_inertia_and_sources() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!((uri, parent), ("mass", "world.sdf"));
            let row = mass_override::cases()
                .into_iter()
                .find(|row| row["name"] == "unit_box_four")
                .unwrap();
            Ok(Resource {
                canonical_uri: "models/mass.sdf".into(),
                xml: row["xml"].as_str().unwrap().into(),
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>mass</uri><name>imported</name><pose>1 2 3 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    let link = &model.links[0];
    assert_eq!(link.inertial.mass, 4.);
    assert_eq!(link.inertial.pose, Pose::default());
    assert_eq!(link.collisions[0].pose, Pose::default());
    for source in [&model.source, &link.source, &link.collisions[0].source] {
        assert_eq!(source.uri, "models/mass.sdf");
    }
    assert_eq!(
        sdf.pose_of("w::imported::l", "::").unwrap().translation,
        [1., 2., 3.]
    );
    assert_eq!(
        sdf.resolve_pose(&link.inertial.pose, "w::imported", "w::imported::l", "::")
            .unwrap()
            .translation,
        [1., 2., 3.]
    );
    let mut copy = model.clone();
    copy.links[0].inertial.mass = 42.;
    assert_eq!(link.inertial.mass, 4.);
    sdf.validate_non_frame_poses().unwrap();
}
