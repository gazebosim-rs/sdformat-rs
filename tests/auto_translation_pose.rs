// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

fn translated_xml(sphere: bool) -> String {
    let (corpus, name) = if sphere {
        (include_str!("auto_sphere_inertial.json"), "explicit_pose")
    } else {
        (include_str!("auto_box_inertial.json"), "pose_translated")
    };
    let data: serde_json::Value = serde_json::from_str(corpus).unwrap();
    let xml = data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .unwrap()["xml"]
        .as_str()
        .unwrap()
        .to_owned();
    if sphere {
        xml.replace("<pose>0 0 0 0 0 0</pose>", "<pose>1 0 0 0 0 0</pose>")
    } else {
        xml
    }
}

#[test]
fn native_nonfinite_accumulator_is_refused_without_returning_a_dom() {
    for sphere in [false, true] {
        for position in ["1e200 0 0", "1.7976931348623157e308 0 0", "1e154 1e154 0"] {
            let xml = translated_xml(sphere).replace(
                "<pose>1 0 0 0 0 0</pose>",
                &format!("<pose>{position} 0 0 0</pose>"),
            );
            let error = load_str(&xml, "aggregate-overflow.sdf", &NoResources).unwrap_err();
            assert_eq!(error.code, ErrorCode::Unsupported);
            assert_eq!(error.source.uri, "aggregate-overflow.sdf");
            assert!(error.source.line > 0 && error.source.column > 0);
        }
    }
}

#[test]
fn translated_unit_center_preserves_density_presence_and_manual_policy() {
    for sphere in [false, true] {
        let original = translated_xml(sphere);
        let density = if sphere {
            "<density>2170</density>"
        } else {
            "<density>1240</density>"
        };
        for (replacement, presence) in [
            ("", None),
            ("<density/>", Some(1000.)),
            ("<density> </density>", Some(1000.)),
            ("<density>1000</density>", Some(1000.)),
        ] {
            let xml = original.replace(density, replacement);
            let origin = load_str(
                &xml.replace("<pose>1 0 0 0 0 0</pose>", ""),
                "origin.sdf",
                &NoResources,
            )
            .unwrap();
            let sdf = load_str(&xml, "translated.sdf", &NoResources).unwrap();
            let link = &sdf.models[0].links[0];
            assert_eq!(link.collisions[0].density, presence);
            assert_eq!(link.collisions[0].density(), 1000.);
            assert_eq!(link.collisions[0].pose.transform.translation, [1., 0., 0.]);
            assert_eq!(link.inertial.pose.transform.translation, [1., 0., 0.]);
            assert_eq!(link.inertial.mass, origin.models[0].links[0].inertial.mass);
            assert_eq!(
                link.inertial.inertia,
                origin.models[0].links[0].inertial.inertia
            );
            sdf.validate_non_frame_poses().unwrap();
            let manual = load_str(
                &xml.replace("auto='true'", "auto='false'"),
                "manual.sdf",
                &NoResources,
            )
            .unwrap();
            let inertial = &manual.models[0].links[0].inertial;
            let default = Inertial::default();
            assert_eq!(inertial.mass, default.mass);
            assert_eq!(inertial.inertia, default.inertia);
            assert_eq!(inertial.pose, default.pose);
            assert_eq!(inertial.fluid_added_mass, default.fluid_added_mass);
            assert_eq!(manual.models[0].links[0].collisions[0].density, presence);
            assert_eq!(
                manual.models[0].links[0].collisions[0]
                    .pose
                    .transform
                    .translation,
                [1., 0., 0.]
            );
        }
    }
}

#[test]
fn translated_import_and_clone_keep_local_center_and_source() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!((uri, parent), ("translated", "world.sdf"));
            Ok(Resource {
                canonical_uri: "boxes/translated.sdf".into(),
                xml: translated_xml(false),
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>translated</uri><name>imported</name><pose>1 2 3 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
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
        [2., 2., 3.]
    );
    for source in [&model.source, &link.source, &link.collisions[0].source] {
        assert_eq!(source.uri, "boxes/translated.sdf");
    }
    let mut copy = model.clone();
    copy.links[0].inertial.pose.transform.translation = [4., 5., 6.];
    assert_eq!(link.inertial.pose.transform.translation, [1., 0., 0.]);
    sdf.validate_non_frame_poses().unwrap();
}

fn native_manual_policy(values: &[u64]) -> Result<Sdf, Diagnostic> {
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

fn four_ulps(actual: f64, expected: u64) {
    let native = f64::from_bits(expected);
    assert!(actual.is_finite() && native.is_finite());
    if actual == native {
        return;
    }
    assert_eq!(actual.is_sign_negative(), native.is_sign_negative());
    assert!(
        actual.to_bits().abs_diff(expected) <= 4,
        "{actual:e} != {native:e}"
    );
}

#[test]
fn captured_native_aggregates_and_existing_manual_policy_are_both_honored() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("auto_translation_pose_cases.json")).unwrap();
    let native: serde_json::Value =
        serde_json::from_str(include_str!("auto_translation_pose_native.json")).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        cases.len(),
        "Native characterization must be completed and reviewed; missing rows cannot pass"
    );
    assert!(!rows.is_empty());
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
        let result = load_str(
            case["xml"].as_str().unwrap(),
            case["uri"].as_str().unwrap(),
            &NoResources,
        );
        let states = row["root_status"].as_array().unwrap();
        assert_eq!(states.len(), 2);
        if states.iter().any(|state| state[0] == 0) {
            let error = result.unwrap_err();
            assert_eq!(error.code, ErrorCode::Unsupported, "{}", case["name"]);
            assert_eq!(error.source.uri, case["uri"].as_str().unwrap());
            continue;
        }
        let snapshots = row["root_values"].as_array().unwrap();
        assert_eq!(snapshots.len(), 2);
        let expected: Vec<u64> = snapshots[1]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap())
            .collect();
        assert_eq!(expected.len(), 18);
        native_manual_policy(&expected).unwrap_or_else(|error| {
            panic!(
                "{} finite native tensor violates the inherited manual policy: {error}",
                case["name"]
            )
        });
        assert!(states.iter().all(|state| state[1] == 1));
        assert!(row["load_errors"].as_array().unwrap().is_empty());
        assert!(row["resolve_errors"].as_array().unwrap().is_empty());
        let sdf = result.unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
        let inertial = &sdf.models[0].links[0].inertial;
        let values = std::iter::once(inertial.mass)
            .chain(inertial.inertia)
            .chain(inertial.pose.transform.translation)
            .chain(inertial.pose.transform.rotation);
        for (actual, expected) in values.zip(&expected[2..16]) {
            four_ulps(actual, *expected);
        }
        sdf.validate_non_frame_poses().unwrap();
    }
}
