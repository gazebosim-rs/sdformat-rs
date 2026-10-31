// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
pub fn assert_empty_flag_model(sdf: &sdformat::Sdf, xml: &str, uri: &str, tag: &str) {
    use sdformat::{LinkProperties, Model, Pose, Source, Transform};
    assert_eq!(sdf.version, "1.12");
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
        source,
    } = &sdf.models[0];
    assert_eq!(name, "m");
    assert_eq!(scoped_name, "m");
    assert_eq!(raw_namespace, &None);
    assert_eq!(pose, &Pose::default());
    assert!(!is_static && !self_collide && *allow_auto_disable && !enable_wind);
    assert!(extensions.is_empty());
    let element = loaded_element.as_ref().unwrap();
    assert_eq!(
        element
            .children()
            .iter()
            .map(|c| (c.tag(), c.name_attribute()))
            .collect::<Vec<_>>(),
        [("link", Some("l")), (tag, None)]
    );
    assert!(canonical_link.is_empty() && placement_frame.is_empty());
    assert!(include.is_none() && merged_includes.is_empty());
    assert!(joints.is_empty() && models.is_empty() && frames.is_empty() && plugins.is_empty());
    assert_eq!(
        source,
        &Source {
            uri: uri.into(),
            line: 1,
            column: xml.find("<model").unwrap() as u32 + 1
        }
    );
    assert_eq!(links.len(), 1);
    let sdformat::Link {
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
        source,
    } = &links[0];
    assert_eq!(name, "l");
    assert_eq!(scoped_name, "m::l");
    assert_eq!(pose, &Pose::default());
    assert_eq!(inertial.mass, 1.);
    assert_eq!(inertial.inertia, [1., 1., 1., 0., 0., 0.]);
    assert_eq!(inertial.pose, Pose::default());
    assert_eq!(inertial.fluid_added_mass, None);
    assert!(collisions.is_empty() && visuals.is_empty() && lights.is_empty());
    assert!(particle_emitters.is_empty() && projectors.is_empty() && sensors.is_empty());
    assert_eq!(properties, &LinkProperties::default());
    assert_eq!(velocity_decay, &None);
    assert!(plugins.is_empty());
    assert_eq!(
        source,
        &Source {
            uri: uri.into(),
            line: 1,
            column: xml.find("<link").unwrap() as u32 + 1
        }
    );
    assert_eq!(sdf.pose_of("m::l", "::").unwrap(), Transform::default());
    assert_eq!(sdf.attached_body("m").unwrap(), "m::l");
    sdf.validate_non_frame_poses().unwrap();
}
