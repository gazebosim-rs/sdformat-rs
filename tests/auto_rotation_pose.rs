// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "common/auto_rigid_pose.rs"]
mod rigid;
#[path = "common/auto_rotation_pose.rs"]
mod rotation;
use sdformat::*;

fn original(sphere: bool) -> String {
    let corpus = if sphere {
        include_str!("auto_sphere_inertial.json")
    } else {
        include_str!("auto_box_inertial.json")
    };
    let name = if sphere {
        "explicit_pose"
    } else {
        "pose_identity"
    };
    let data: serde_json::Value = serde_json::from_str(corpus).unwrap();
    data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .unwrap()["xml"]
        .as_str()
        .unwrap()
        .replace("<pose>0 0 0 0 0 0</pose>", "<pose>0 0 0 0.1 0 0</pose>")
}

#[test]
fn origin_rotation_preserves_density_presence_manual_defaults_and_tensor() {
    for sphere in [false, true] {
        let base = original(sphere);
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
            let xml = base.replace(density, replacement);
            let sdf = load_str(&xml, "density-rotation.sdf", &NoResources).unwrap();
            let link = &sdf.models[0].links[0];
            assert_eq!(link.collisions[0].density, presence);
            assert_eq!(link.collisions[0].density(), 1000.);
            assert_eq!(link.inertial.pose, Pose::default());
            assert!(link.inertial.inertia.iter().all(|value| value.is_finite()));
            let manual = load_str(
                &xml.replace("auto='true'", "auto='false'"),
                "density-rotation.sdf",
                &NoResources,
            )
            .unwrap();
            let inertial = &manual.models[0].links[0].inertial;
            let default = Inertial::default();
            assert_eq!(inertial.mass, default.mass);
            assert_eq!(inertial.inertia, default.inertia);
            assert_eq!(inertial.pose, default.pose);
            assert_eq!(inertial.fluid_added_mass, default.fluid_added_mass);
            assert_eq!(
                manual.models[0].links[0].collisions[0].pose,
                link.collisions[0].pose
            );
            assert_eq!(manual.models[0].links[0].collisions[0].density, presence);
            sdf.validate_non_frame_poses().unwrap();
        }
    }
}

#[test]
fn mixed_transforms_attributes_and_overrides_remain_closed() {
    for sphere in [false, true] {
        let original = original(sphere);
        for pose in [
            "<pose>1 0 0 0.1 0 0</pose>",
            "<pose relative_to=''>0 0 0 0.1 0 0</pose>",
            "<pose relative_to='link'>0 0 0 0.1 0 0</pose>",
            "<pose degrees='false'>0 0 0 0.1 0 0</pose>",
            "<pose rotation_format='euler_rpy'>0 0 0 0.1 0 0</pose>",
            "<pose rotation_format='quat_xyzw'>0 0 0 0 0 0 1</pose>",
        ] {
            let xml = original.replace("<pose>0 0 0 0.1 0 0</pose>", pose);
            if pose == "<pose>1 0 0 0.1 0 0</pose>" {
                rigid::assert_translated_rotation(&xml, "closed-rotation.sdf", sphere);
            } else {
                let error = load_str(&xml, "closed-rotation.sdf", &NoResources).unwrap_err();
                assert_eq!(error.code, ErrorCode::Unsupported);
                assert_eq!(error.source.uri, "closed-rotation.sdf");
            }
        }
        let overridden = original.replace(
            "<inertial auto='true'/>",
            "<inertial auto='true'><mass>1</mass></inertial>",
        );
        assert_eq!(
            load_str(&overridden, "override.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        let multiple = original.replace("</link>", "<collision name='second'/></link>");
        assert_eq!(
            load_str(&multiple, "multiple.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}

#[test]
fn rotated_import_and_clone_keep_local_tensor_and_resource_source() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!((uri, parent), ("rotated", "world.sdf"));
            Ok(Resource {
                canonical_uri: "shapes/rotated.sdf".into(),
                xml: original(false),
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>rotated</uri><name>imported</name><pose>1 2 3 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    let link = &model.links[0];
    assert_eq!(link.inertial.pose, Pose::default());
    assert_eq!(
        sdf.resolve_pose(
            &link.inertial.pose,
            "w::imported",
            "w::imported::link",
            "::"
        )
        .unwrap()
        .translation,
        [1., 2., 3.]
    );
    for source in [&model.source, &link.source, &link.collisions[0].source] {
        assert_eq!(source.uri, "shapes/rotated.sdf");
    }
    let expected = link.inertial.inertia;
    let mut copied = model.clone();
    copied.links[0].inertial.inertia = [2.; 6];
    copied.links[0].collisions[0].pose = Pose::default();
    assert_eq!(link.inertial.inertia, expected);
    assert_ne!(link.collisions[0].pose, Pose::default());
    sdf.validate_non_frame_poses().unwrap();
}

fn manual_policy(values: &[u64]) -> Result<Sdf, Diagnostic> {
    let value = |i| f64::from_bits(values[i]).to_string();
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='l'><inertial><mass>{}</mass><inertia><ixx>{}</ixx><iyy>{}</iyy><izz>{}</izz><ixy>{}</ixy><ixz>{}</ixz><iyz>{}</iyz></inertia></inertial></link></model></sdf>",
        value(2),
        value(3),
        value(4),
        value(5),
        value(6),
        value(7),
        value(8)
    );
    load_str(&xml, "manual-policy.sdf", &NoResources)
}

#[test]
fn reviewed_native_rotation_data_preserves_values_or_existing_numeric_refusals() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("auto_rotation_pose_cases.json")).unwrap();
    let native: serde_json::Value =
        serde_json::from_str(include_str!("auto_rotation_pose_native.json")).unwrap();
    let cases = cases["cases"].as_array().unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert!(
        !rows.is_empty(),
        "Actual reviewed native capture is required"
    );
    assert_eq!(cases.len(), rows.len());
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
        let matrix = row["collision_moi"].as_array().unwrap();
        assert_eq!(matrix.len(), 9);
        assert!(
            matrix
                .iter()
                .all(|value| f64::from_bits(value.as_u64().unwrap()).is_finite())
        );
        let aggregate = row["root_values"][1].as_array().unwrap();
        assert_eq!(aggregate.len(), 18);
        for (index, expected) in [0, 4, 8, 1, 2, 5].into_iter().zip(&aggregate[3..9]) {
            assert_eq!(
                f64::from_bits(matrix[index].as_u64().unwrap()),
                f64::from_bits(expected.as_u64().unwrap()),
                "{} direct Moi upper triangle must feed the origin aggregate",
                case["name"]
            );
        }
        if case["name"] == "box_old_rotation" || case["name"] == "sphere_old_rotation" {
            rotation::assert_origin_rotation(
                case["xml"].as_str().unwrap(),
                case["uri"].as_str().unwrap(),
                case["name"] == "sphere_old_rotation",
            );
            continue;
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
        assert_eq!(row["root_status"][1][1], 1);
        let values: Vec<u64> = row["root_values"][1]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_u64().unwrap())
            .collect();
        if let Err(error) = manual_policy(&values) {
            assert_eq!(error.code, ErrorCode::InvalidValue);
            assert_eq!(
                result.unwrap_err().code,
                ErrorCode::InvalidValue,
                "{}",
                case["name"]
            );
            continue;
        }
        let sdf = result.unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
        let inertial = &sdf.models[0].links[0].inertial;
        for (actual, expected) in std::iter::once(inertial.mass)
            .chain(inertial.inertia)
            .zip(&values[2..9])
        {
            rotation::four_ulps(actual, *expected);
        }
        assert_eq!(inertial.pose, Pose::default());
        assert!(row["collision_moi"].as_array().unwrap().len() == 9);
        sdf.validate_non_frame_poses().unwrap();
    }
}
