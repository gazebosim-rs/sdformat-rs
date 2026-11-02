// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

fn native_rotation(name: &str) -> [u64; 4] {
    let native: serde_json::Value =
        serde_json::from_str(include_str!("auto_rotation_pose_native.json")).unwrap();
    let row = native["observations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .unwrap();
    std::array::from_fn(|index| row["collision_values"][10 + index].as_u64().unwrap())
}

fn assert_native_rotation(pose: &Pose, bits: [u64; 4]) {
    for (actual, expected) in pose.transform.rotation.into_iter().zip(bits) {
        if f64::from_bits(expected) == 0. {
            // The captured pose was multiplied by identity. That preserves
            // nonzero components, but does not qualify signed-zero bits.
            assert_eq!(actual, 0.);
        } else {
            assert_eq!(actual.to_bits(), expected);
        }
    }
}

#[test]
fn native_radian_components_reuse_every_captured_rotation_input() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("auto_rotation_pose_cases.json")).unwrap();
    let native: serde_json::Value =
        serde_json::from_str(include_str!("auto_rotation_pose_native.json")).unwrap();
    let cases = cases["cases"].as_array().unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    assert_eq!(cases.len(), rows.len());
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
        // Raw Euler parsing is independent of automatic-inertia admission.
        let xml = case["xml"]
            .as_str()
            .unwrap()
            .replace("auto='true'", "auto='false'");
        let uri = case["uri"].as_str().unwrap();
        let sdf = load_str(&xml, uri, &NoResources).unwrap();
        assert_eq!(sdf.version, "1.12");
        assert!(sdf.worlds.is_empty());
        assert_eq!(sdf.models.len(), 1);
        let model = &sdf.models[0];
        assert_eq!(model.links.len(), 1);
        let link = &model.links[0];
        assert_eq!(link.collisions.len(), 1);
        let collision = &link.collisions[0];
        let bits =
            std::array::from_fn(|index| row["collision_values"][10 + index].as_u64().unwrap());
        assert_native_rotation(&collision.pose, bits);
        assert_eq!(collision.pose.transform.translation, [0.; 3]);
        assert!(collision.pose.relative_to.is_empty());
        assert_eq!(model.pose, Pose::default());
        assert_eq!(link.pose, Pose::default());
        assert_eq!(link.inertial.mass, 1.);
        assert_eq!(link.inertial.inertia, [1., 1., 1., 0., 0., 0.]);
        assert_eq!(link.inertial.pose, Pose::default());
        assert!(link.inertial.fluid_added_mass.is_none());
        for (source, tag) in [
            (&model.source, "model"),
            (&link.source, "link"),
            (&collision.source, "collision"),
        ] {
            assert_eq!(source.uri, uri);
            assert_eq!(source.line, 1);
            assert_eq!(
                source.column as usize,
                xml.find(&format!("<{tag}")).unwrap() + 1
            );
        }
        assert_eq!(collision.clone().pose, collision.pose);
        assert_eq!(collision.clone().source, collision.source);
        assert_eq!(
            sdf.resolve_pose(
                &collision.pose,
                &model.scoped_name,
                &link.scoped_name,
                &link.scoped_name
            )
            .unwrap(),
            collision.pose.transform
        );
        collision
            .pose
            .transform
            .validate(&collision.source)
            .unwrap();
    }
}

#[test]
fn radian_constructor_is_shared_without_losing_member_frames_or_sources() {
    let xml = concat!(
        "<sdf version='1.12'><world name='w'><model name='m'><pose>1 2 3 0.1 0.2 0.3</pose>",
        "<frame name='f' attached_to='l'><pose relative_to='l'>1 2 3 0.1 0.2 0.3</pose></frame>",
        "<link name='l'><pose>1 2 3 0.1 0.2 0.3</pose><inertial><pose>1 2 3 0.1 0.2 0.3</pose></inertial>",
        "<collision name='c'><pose relative_to='f'>1 2 3 0.1 0.2 0.3</pose><geometry><box/></geometry></collision>",
        "<sensor name='s' type='imu'><pose>1 2 3 0.1 0.2 0.3</pose></sensor></link>",
        "<joint name='j' type='fixed'><parent>world</parent><child>l</child><pose relative_to='l'>1 2 3 0.1 0.2 0.3</pose></joint>",
        "</model><light name='lamp' type='point'><pose>1 2 3 0.1 0.2 0.3</pose></light></world></sdf>"
    );
    let sdf = load_str(xml, "members.sdf", &NoResources).unwrap();
    let world = &sdf.worlds[0];
    let model = &world.models[0];
    let link = &model.links[0];
    let collision = &link.collisions[0];
    let bits = native_rotation("box_mixed");
    for (pose, relative_to) in [
        (&model.pose, ""),
        (&model.frames[0].pose, "l"),
        (&link.pose, ""),
        (&link.inertial.pose, ""),
        (&collision.pose, "f"),
        (&link.sensors[0].pose, ""),
        (&model.joints[0].pose, "l"),
        (&world.lights[0].pose, ""),
    ] {
        assert_native_rotation(pose, bits);
        assert_eq!(pose.transform.translation, [1., 2., 3.]);
        assert_eq!(pose.relative_to, relative_to);
    }
    for source in [
        &world.source,
        &model.source,
        &link.source,
        &collision.source,
        &model.frames[0].source,
        &link.sensors[0].source,
        &model.joints[0].source,
        world.lights[0]
            .source
            .as_ref()
            .expect("loaded light has source"),
    ] {
        assert_eq!(source.uri, "members.sdf");
        assert_eq!(source.line, 1);
        assert!(source.column > 0);
    }
    assert_eq!(format!("{:?}", model.clone()), format!("{model:?}"));
    assert_eq!(
        sdf.pose_of("w::m::f", "w::m::f").unwrap(),
        Transform::default()
    );
    assert_eq!(sdf.attached_body("w::m::f").unwrap(), "w::m::l");
    assert_eq!(
        sdf.resolve_pose(&collision.pose, "w::m", "w::m::l", "w::m::f")
            .unwrap(),
        collision.pose.transform
    );
    sdf.validate_non_frame_poses().unwrap();
}

#[test]
fn radian_spellings_defaults_and_deferred_unknown_frames_keep_their_contract() {
    for attributes in [
        "",
        " degrees='false'",
        " degrees='0'",
        " rotation_format='euler_rpy'",
    ] {
        for angles in ["0.1 0.2 0.3", "1e-1 2e-1 3e-1"] {
            let xml = format!(
                "<sdf version='1.12'><model name='m'><pose{attributes}>1 2 3 {angles}</pose><link name='l'/></model></sdf>"
            );
            let sdf = load_str(&xml, "spellings.sdf", &NoResources).unwrap();
            assert_native_rotation(&sdf.models[0].pose, native_rotation("box_mixed"));
            assert_eq!(sdf.models[0].pose.transform.translation, [1., 2., 3.]);
        }
    }
    for pose in [
        "",
        "<pose/>",
        "<pose> </pose>",
        "<pose>0 0 0 0 0 0</pose>",
        "<pose>-0 -0 -0 -0 -0 -0</pose>",
    ] {
        let xml =
            format!("<sdf version='1.12'><model name='m'>{pose}<link name='l'/></model></sdf>");
        let sdf = load_str(&xml, "identity.sdf", &NoResources).unwrap();
        assert_eq!(sdf.models[0].pose, Pose::default());
    }
    let xml = "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><pose relative_to='missing'>0 0 0 0.1 0.2 0.3</pose><geometry><box/></geometry></collision></link></model></sdf>";
    let sdf = load_str(xml, "deferred.sdf", &NoResources).unwrap();
    let pose = &sdf.models[0].links[0].collisions[0].pose;
    assert_native_rotation(pose, native_rotation("box_mixed"));
    assert_eq!(pose.relative_to, "missing");
    assert_eq!(
        sdf.validate_non_frame_poses().unwrap_err().code,
        ErrorCode::UnknownReference
    );
}

#[test]
fn degree_quaternion_and_malformed_pose_paths_remain_unchanged() {
    let xml = "<sdf version='1.12'><model name='m'><pose degrees='true'>1 2 3 13 17 23</pose><link name='l'/></model></sdf>";
    let sdf = load_str(xml, "degrees.sdf", &NoResources).unwrap();
    let factor = std::f64::consts::PI / 180.;
    let expected = glam::DQuat::from_euler(
        glam::EulerRot::ZYX,
        23. * factor,
        17. * factor,
        13. * factor,
    );
    assert_eq!(
        sdf.models[0].pose.transform.rotation.map(f64::to_bits),
        expected.to_array().map(f64::to_bits)
    );
    let xml = "<sdf version='1.12'><model name='m'><pose rotation_format='quat_xyzw'>1 2 3 1 2 3 4</pose><link name='l'/></model></sdf>";
    let sdf = load_str(xml, "quaternion.sdf", &NoResources).unwrap();
    assert_eq!(
        sdf.models[0].pose.transform.rotation.map(f64::to_bits),
        glam::DQuat::from_xyzw(1., 2., 3., 4.)
            .normalize()
            .to_array()
            .map(f64::to_bits)
    );
    for pose in [
        "<pose>0 0 0 0 0</pose>",
        "<pose>0 0 0 NaN 0 0</pose>",
        "<pose degrees='maybe'>0 0 0 0 0 0</pose>",
        "<pose rotation_format='bad'>0 0 0 0 0 0</pose>",
        "<pose rotation_format='quat_xyzw'>0 0 0 0 0 0 0</pose>",
        "<pose degrees='true' rotation_format='quat_xyzw'>0 0 0 0 0 0 1</pose>",
    ] {
        let xml =
            format!("<sdf version='1.12'><model name='m'>{pose}<link name='l'/></model></sdf>");
        let error = load_str(&xml, "invalid-pose.sdf", &NoResources).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidValue);
        assert_eq!(error.source.uri, "invalid-pose.sdf");
    }
}

#[test]
fn included_radian_pose_keeps_the_resolver_source() {
    struct Resolver;
    impl ResourceResolver for Resolver {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!(uri, "part");
            assert_eq!(parent, "outer.sdf");
            Ok(Resource { canonical_uri: "fixture:radian-euler".into(), xml: "<sdf version='1.12'><model name='payload'><pose>1 2 3 0.1 0.2 0.3</pose><link name='l'/></model></sdf>".into() })
        }
    }
    let sdf = load_str(
        "<sdf version='1.12'><world name='w'><include><uri>part</uri></include></world></sdf>",
        "outer.sdf",
        &Resolver,
    )
    .unwrap();
    let model = &sdf.worlds[0].models[0];
    assert_eq!(model.scoped_name, "w::payload");
    assert_eq!(model.source.uri, "fixture:radian-euler");
    assert_eq!(model.links[0].source.uri, "fixture:radian-euler");
    assert_native_rotation(&model.pose, native_rotation("box_mixed"));
    assert_eq!(model.pose.transform.translation, [1., 2., 3.]);
    assert_eq!(model.clone().source, model.source);
}
