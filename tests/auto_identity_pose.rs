// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "common/auto_default_pose_attributes.rs"]
mod attributes;
#[path = "common/auto_rotation_pose.rs"]
mod rotation;
use sdformat::*;

fn original_row(is_sphere: bool) -> serde_json::Value {
    let (data, name) = if is_sphere {
        (include_str!("auto_sphere_inertial.json"), "explicit_pose")
    } else {
        (include_str!("auto_box_inertial.json"), "pose_identity")
    };
    let corpus: serde_json::Value = serde_json::from_str(data).unwrap();
    corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .unwrap()
        .clone()
}

#[path = "common/auto_translation_pose.rs"]
mod translation;

#[test]
fn existing_euler_identity_values_do_not_depend_on_decimal_spelling() {
    for is_sphere in [false, true] {
        let row = original_row(is_sphere);
        let original = row["xml"].as_str().unwrap();
        let control = load_str(original, "identity.sdf", &NoResources).unwrap();
        let expected = &control.models[0].links[0].inertial;
        for pose in [
            "<pose>0 0 0 0 0 0</pose>",
            "<pose>0.0 +0.00 0e0 0E+0 0.000 0.0e-3</pose>",
            "<pose> \t0 0 0 0 0 0\n </pose>",
            "<pose/>",
        ] {
            let xml = original.replace("<pose>0 0 0 0 0 0</pose>", pose);
            let sdf = load_str(&xml, "identity.sdf", &NoResources).unwrap();
            let link = &sdf.models[0].links[0];
            assert_eq!(link.collisions[0].pose, Pose::default());
            assert_eq!(link.inertial.mass, expected.mass);
            assert_eq!(link.inertial.inertia, expected.inertia);
            assert_eq!(link.inertial.pose, expected.pose);
            assert_eq!(link.inertial.fluid_added_mass, expected.fluid_added_mass);
            assert_eq!(link.collisions[0].source.uri, "identity.sdf");
            sdf.validate_non_frame_poses().unwrap();
        }
    }
}

#[test]
fn transformed_and_attributed_pose_families_stay_explicitly_unsupported() {
    for is_sphere in [false, true] {
        let row = original_row(is_sphere);
        let original = row["xml"].as_str().unwrap();
        for pose in [
            "<pose>1 0 0 0 0 0</pose>",
            "<pose>0 0 0 0.1 0 0</pose>",
            "<pose degrees='true'>0 0 0 0 0 0</pose>",
            "<pose rotation_format='euler_rpy'>0 0 0 0 0 0</pose>",
            "<pose rotation_format='quat_xyzw'>0 0 0 0 0 0 1</pose>",
            "<pose relative_to=''>0 0 0 0 0 0</pose>",
            "<pose relative_to='missing'>0 0 0 0 0 0</pose>",
            "<pose relative_to='__model__'>0 0 0 0 0 0</pose>",
        ] {
            let xml = original.replace("<pose>0 0 0 0 0 0</pose>", pose);
            if pose == "<pose>1 0 0 0 0 0</pose>" {
                translation::assert_unit_translation(&xml, "closed-pose.sdf", is_sphere);
                continue;
            }
            if pose == "<pose>0 0 0 0.1 0 0</pose>" {
                rotation::assert_origin_rotation(&xml, "closed-pose.sdf", is_sphere);
                continue;
            }
            if pose == "<pose rotation_format='euler_rpy'>0 0 0 0 0 0</pose>" {
                attributes::assert_default_attributes(
                    &xml,
                    "closed-pose.sdf",
                    if is_sphere {
                        "sphere_old_identity"
                    } else {
                        "box_old_identity"
                    },
                );
                continue;
            }
            let error = load_str(&xml, "closed-pose.sdf", &NoResources).unwrap_err();
            assert_eq!(error.code, ErrorCode::Unsupported, "{pose}");
            assert_eq!(error.source.uri, "closed-pose.sdf");
            assert!(error.source.line > 0 && error.source.column > 0);
        }
        for override_element in ["<pose/>", "<mass>1</mass>"] {
            let xml = original.replace(
                "<inertial auto='true'/>",
                &format!("<inertial auto='true'>{override_element}</inertial>"),
            );
            assert_eq!(
                load_str(&xml, "override.sdf", &NoResources)
                    .unwrap_err()
                    .code,
                ErrorCode::Unsupported
            );
        }
        let xml = original.replace("</link>", "<collision name='second'><density>1</density><geometry><sphere><radius>1</radius></sphere></geometry></collision></link>");
        assert_eq!(
            load_str(&xml, "multiple.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}

#[test]
fn identity_pose_clone_and_import_keep_typed_pose_and_resource_source() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!(uri, "box");
            assert_eq!(parent, "world.sdf");
            Ok(Resource {
                canonical_uri: "boxes/identity.sdf".into(),
                xml: original_row(false)["xml"].as_str().unwrap().to_owned(),
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>box</uri><name>imported</name><pose>1 2 3 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    let link = &model.links[0];
    let collision = &link.collisions[0];
    assert_eq!(collision.pose, Pose::default());
    assert_eq!(link.inertial.pose, Pose::default());
    assert_eq!(link.inertial.mass.to_bits(), 4666679192002232320);
    for source in [&model.source, &link.source, &collision.source] {
        assert_eq!(source.uri, "boxes/identity.sdf");
    }
    assert_eq!(
        sdf.pose_of("w::imported::link", "::").unwrap().translation,
        [1., 2., 3.]
    );
    let mut copy = model.clone();
    copy.links[0].collisions[0].pose.transform.translation = [1., 2., 3.];
    copy.links[0].collisions[0].source.uri = "edited.sdf".into();
    assert_eq!(collision.pose, Pose::default());
    assert_eq!(collision.source.uri, "boxes/identity.sdf");
    assert_eq!(link.inertial.mass.to_bits(), 4666679192002232320);
    sdf.validate_non_frame_poses().unwrap();
}

#[test]
fn identity_pose_keeps_inherited_density_fallback_and_presence() {
    for is_sphere in [false, true] {
        let row = original_row(is_sphere);
        let original = row["xml"].as_str().unwrap();
        let old_density = if is_sphere {
            "<density>2170</density>"
        } else {
            "<density>1240</density>"
        };
        for (density_xml, presence) in [
            ("", None),
            ("<density/>", Some(1000.)),
            ("<density> </density>", Some(1000.)),
            ("<density>1000</density>", Some(1000.)),
        ] {
            assert!(original.contains(old_density));
            let xml = original.replace(old_density, density_xml);
            let omitted_pose = xml.replace("<pose>0 0 0 0 0 0</pose>", "");
            let control = load_str(&omitted_pose, "density-control.sdf", &NoResources).unwrap();
            let sdf = load_str(&xml, "density-identity.sdf", &NoResources).unwrap();
            let link = &sdf.models[0].links[0];
            let collision = &link.collisions[0];
            let expected = &control.models[0].links[0];
            assert_eq!(collision.density, presence);
            assert_eq!(collision.density(), 1000.);
            assert_eq!(collision.geometry, expected.collisions[0].geometry);
            assert_eq!(collision.pose, Pose::default());
            assert_eq!(link.inertial.mass, expected.inertial.mass);
            assert_eq!(link.inertial.inertia, expected.inertial.inertia);
            assert_eq!(link.inertial.pose, expected.inertial.pose);
            assert_eq!(
                link.inertial.fluid_added_mass,
                expected.inertial.fluid_added_mass
            );
            assert_eq!(collision.source.uri, "density-identity.sdf");
            assert_eq!(
                sdf.pose_of("shapes::link", "::").unwrap(),
                Transform::default()
            );
            sdf.validate_non_frame_poses().unwrap();
        }
    }
}
