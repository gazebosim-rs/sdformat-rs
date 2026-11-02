// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "common/auto_default_pose_attributes.rs"]
mod attributes;
use sdformat::*;

#[test]
fn default_euler_attributes_preserve_actual_native_values_and_complete_owned_dom() {
    let cases = attributes::cases();
    assert_eq!(cases.len(), 24);
    assert_eq!(
        cases
            .iter()
            .filter(|row| !row["old_test"].is_null())
            .count(),
        16
    );
    for case in cases {
        attributes::assert_default_attributes(
            case["xml"].as_str().unwrap(),
            case["uri"].as_str().unwrap(),
            case["name"].as_str().unwrap(),
        );
    }
}

fn replace_pose(xml: &str, pose: &str) -> String {
    let start = xml.find("<pose>").unwrap();
    let end = xml[start..].find("</pose>").unwrap() + start + "</pose>".len();
    format!("{}{pose}{}", &xml[..start], &xml[end..])
}

#[test]
fn frames_degree_quaternion_invalid_and_capsule_rotation_families_remain_closed() {
    for case in attributes::cases()
        .into_iter()
        .filter(|row| !row["old_test"].is_null())
    {
        let xml = case["base_xml"].as_str().unwrap();
        for attribute in [
            "relative_to=''",
            "relative_to='missing'",
            "degrees='true'",
            "degrees='1'",
            "frame=''",
        ] {
            let input = xml.replace("<pose>", &format!("<pose {attribute}>"));
            let error =
                load_str(&input, "closed-default-attributes.sdf", &NoResources).unwrap_err();
            assert_eq!(error.code, ErrorCode::Unsupported, "{attribute}");
            assert_eq!(error.source.uri, "closed-default-attributes.sdf");
        }
        let quaternion = replace_pose(
            xml,
            "<pose rotation_format='quat_xyzw'>0 0 0 0 0 0 1</pose>",
        );
        assert_eq!(
            load_str(&quaternion, "quaternion.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
        for attribute in ["degrees='sometimes'", "rotation_format='unknown'"] {
            let input = xml.replace("<pose>", &format!("<pose {attribute}>"));
            assert_eq!(
                load_str(&input, "invalid-attribute.sdf", &NoResources)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidValue
            );
        }
        let xml = case["xml"].as_str().unwrap();
        for content in ["<mass>1</mass>", "<pose/>", "<inertia/>"] {
            let input = xml.replace(
                "<inertial auto='true'/>",
                &format!("<inertial auto='true'>{content}</inertial>"),
            );
            assert_eq!(
                load_str(&input, "override.sdf", &NoResources)
                    .unwrap_err()
                    .code,
                ErrorCode::Unsupported
            );
        }
        let multiple = xml.replace("</link>", "<collision name='second'><geometry><sphere><radius>1</radius></sphere></geometry></collision></link>");
        assert_eq!(
            load_str(&multiple, "multiple.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    let row = attributes::cases()
        .into_iter()
        .find(|row| row["name"] == "capsule_both_false")
        .unwrap();
    let xml = row["base_xml"].as_str().unwrap();
    for pose in [
        "<pose degrees='false'>0 0 0 0.1 0 0</pose>",
        "<pose rotation_format='euler_rpy'>1 0 0 0.1 0 0</pose>",
    ] {
        assert_eq!(
            load_str(
                &replace_pose(xml, pose),
                "rotated-capsule.sdf",
                &NoResources
            )
            .unwrap_err()
            .code,
            ErrorCode::Unsupported
        );
    }
}

#[test]
fn default_attributes_keep_density_presence_and_empty_pose_defaults() {
    for name in [
        "box_both_false",
        "sphere_both_zero",
        "cylinder_zero",
        "capsule_zero",
    ] {
        let row = attributes::cases()
            .into_iter()
            .find(|row| row["name"] == name)
            .unwrap();
        let xml = row["base_xml"].as_str().unwrap();
        let no_density = if let Some(start) = xml.find("<density>") {
            let end = xml[start..].find("</density>").unwrap() + start + "</density>".len();
            format!("{}{}", &xml[..start], &xml[end..])
        } else {
            xml.to_owned()
        };
        for (density, presence) in [
            ("", None),
            ("<density/>", Some(1000.)),
            ("<density> </density>", Some(1000.)),
            ("<density>1000</density>", Some(1000.)),
        ] {
            let base = no_density.replace("<geometry>", &format!("{density}<geometry>"));
            let input = base.replace("<pose>", "<pose degrees='0' rotation_format='euler_rpy'>");
            let plain = load_str(&base, "density.sdf", &NoResources).unwrap();
            let sdf = load_str(&input, "density.sdf", &NoResources).unwrap();
            assert_eq!(format!("{:?}", sdf.models), format!("{:?}", plain.models));
            assert_eq!(sdf.models[0].links[0].collisions[0].density, presence);
        }
        let empty = replace_pose(
            xml,
            "<pose degrees='false' rotation_format='euler_rpy'></pose>",
        );
        let plain = replace_pose(xml, "<pose/>");
        let control = load_str(&plain, "empty-pose.sdf", &NoResources).unwrap();
        let actual = load_str(&empty, "empty-pose.sdf", &NoResources).unwrap();
        assert_eq!(
            format!("{:?}", actual.models),
            format!("{:?}", control.models)
        );
        assert_eq!(
            actual.models[0].links[0].collisions[0].pose,
            Pose::default()
        );
    }
}

#[test]
fn default_attributes_include_keeps_canonical_source_and_local_inertial_frame() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!((uri, parent), ("capsule", "world.sdf"));
            let row = attributes::cases()
                .into_iter()
                .find(|row| row["name"] == "capsule_both_false")
                .unwrap();
            Ok(Resource {
                canonical_uri: "capsules/default-attributes.sdf".into(),
                xml: row["xml"].as_str().unwrap().into(),
            })
        }
    }
    let xml = "<sdf version='1.12'><world name='w'><include><uri>capsule</uri><name>imported</name><pose>4 5 6 0 0 0</pose></include></world></sdf>";
    let sdf = load_str(xml, "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    let link = &model.links[0];
    assert_eq!(model.name, "imported");
    assert_eq!(link.collisions[0].pose.transform.translation, [1., 0., 0.]);
    for source in [&model.source, &link.source, &link.collisions[0].source] {
        assert_eq!(source.uri, "capsules/default-attributes.sdf");
    }
    assert_eq!(
        sdf.pose_of("w::imported::l", "::").unwrap().translation,
        [4., 5., 6.]
    );
    let center = sdf
        .resolve_pose(&link.inertial.pose, "w::imported", "w::imported::l", "::")
        .unwrap();
    for (actual, expected) in center.translation.into_iter().zip(
        link.inertial
            .pose
            .transform
            .translation
            .into_iter()
            .zip([4., 5., 6.])
            .map(|(local, offset)| local + offset),
    ) {
        assert_eq!(actual, expected);
    }
    let mut copy = model.clone();
    copy.links[0].collisions[0].source.uri = "edited.sdf".into();
    assert_eq!(
        link.collisions[0].source.uri,
        "capsules/default-attributes.sdf"
    );
    sdf.validate_non_frame_poses().unwrap();
}
