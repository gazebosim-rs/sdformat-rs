// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

pub fn assert_same_input_positive(sdf: &Sdf) {
    let xml = "<sdf version='1.5'><model name='m'><link name='l'><sensor name='s' type='imu'><imu><noise><type>gaussian</type><rate><mean>2</mean></rate><accel><mean>6</mean></accel></noise></imu></sensor></link></model></sdf>";
    let source = |tag: &str| Source {
        uri: "unsupported-conversion".into(),
        line: 1,
        column: u32::try_from(xml.find(tag).unwrap() + 1).unwrap(),
    };
    assert_eq!(sdf.version, "1.5");
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let Model {
        name,
        scoped_name,
        raw_namespace,
        pose,
        is_static,
        self_collide,
        allow_auto_disable,
        enable_wind,
        extensions,
        loaded_element,
        canonical_link,
        placement_frame,
        include,
        merged_includes,
        links,
        joints,
        models,
        frames,
        plugins,
        source: origin,
    } = &sdf.models[0];
    assert_eq!(name, "m");
    assert_eq!(scoped_name, "m");
    assert_eq!(*raw_namespace, None);
    assert_eq!(*pose, Pose::default());
    assert!(!is_static && !self_collide && *allow_auto_disable && !enable_wind);
    assert!(extensions.is_empty());
    let children = loaded_element.as_ref().unwrap().children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].tag(), "link");
    assert_eq!(children[0].name_attribute(), Some("l"));
    assert!(canonical_link.is_empty() && placement_frame.is_empty());
    assert!(include.is_none() && merged_includes.is_empty());
    assert!(joints.is_empty() && models.is_empty() && frames.is_empty() && plugins.is_empty());
    assert_eq!(*origin, source("<model"));
    assert_eq!(links.len(), 1);
    let Link {
        name,
        scoped_name,
        pose,
        inertial,
        collisions,
        visuals,
        lights,
        particle_emitters,
        projectors,
        sensors,
        properties,
        velocity_decay,
        plugins,
        source: origin,
    } = &links[0];
    assert_eq!(name, "l");
    assert_eq!(scoped_name, "m::l");
    assert_eq!(*pose, Pose::default());
    assert_eq!(inertial.mass, 1.);
    assert_eq!(inertial.inertia, [1., 1., 1., 0., 0., 0.]);
    assert_eq!(inertial.pose, Pose::default());
    assert_eq!(inertial.fluid_added_mass, None);
    assert!(collisions.is_empty() && visuals.is_empty() && lights.is_empty());
    assert!(particle_emitters.is_empty() && projectors.is_empty() && plugins.is_empty());
    assert_eq!(*properties, LinkProperties::default());
    assert_eq!(*velocity_decay, None);
    assert_eq!(*origin, source("<link"));
    assert_eq!(sensors.len(), 1);
    let Sensor {
        name,
        kind,
        pose,
        update_rate,
        always_on,
        visualize,
        extensions,
        enable_metrics,
        config,
        plugins,
        topic,
        frame_id,
        gz_frame_id,
        configuration_xml,
        source: origin,
    } = &sensors[0];
    assert_eq!(name, "s");
    assert_eq!(kind, "imu");
    assert_eq!(*pose, Pose::default());
    assert_eq!(*update_rate, 0.);
    assert!(!always_on && !visualize && !enable_metrics);
    assert!(extensions.is_empty() && plugins.is_empty());
    assert!(topic.is_empty() && frame_id.is_empty());
    assert_eq!(*gz_frame_id, None);
    assert_eq!(
        configuration_xml,
        "<imu><noise><type>gaussian</type><rate><mean>2</mean></rate><accel><mean>6</mean></accel></noise></imu>"
    );
    assert_eq!(*origin, source("<sensor"));
    let SensorConfig::Imu(imu) = config else {
        panic!("IMU payload")
    };
    assert_eq!(
        **imu,
        Imu {
            source: Some(source("<imu>")),
            angular_velocity: [Noise {
                kind: NoiseKind::Gaussian,
                mean: 2.,
                ..Noise::default()
            }; 3],
            linear_acceleration: [Noise {
                kind: NoiseKind::Gaussian,
                mean: 6.,
                ..Noise::default()
            }; 3],
            ..Imu::default()
        }
    );
    sdf.validate_non_frame_poses().unwrap();
    assert_eq!(sdf.pose_of("m::l", "::").unwrap(), Transform::default());
    assert_eq!(
        sdf.resolve_pose(pose, "m", "m::l", "::").unwrap(),
        Transform::default()
    );
    assert_eq!(sdf.attached_body("m::l").unwrap(), "m::l");
}
