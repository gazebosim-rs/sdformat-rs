// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Each integration target uses its relevant assertion helper.
#![allow(dead_code)]
use sdformat::*;
pub fn empty(value: &Geometry) {
    assert_eq!(value, &Geometry::Empty);
    let xml = value.to_xml().unwrap();
    assert_eq!(xml, "<geometry><empty/></geometry>");
    assert_eq!(
        load_geometry_str(&xml, "reload.xml").unwrap(),
        Geometry::Empty
    );
}
pub fn geometry(xml: &str, source: &str) {
    empty(&load_geometry_str(xml, source).unwrap());
}
pub fn visual(xml: &str, source: &str) {
    let value = load_visual_str(xml, source).unwrap();
    assert_eq!(
        value.source,
        Source {
            uri: source.into(),
            line: 1,
            column: 1
        }
    );
    assert_eq!(value.name, "v");
    assert_eq!(value.geometry, Some(Geometry::Empty));
    assert_eq!(
        value.pose.transform,
        Transform {
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.]
        }
    );
    assert!(value.pose.relative_to.is_empty());
    assert_eq!(value.properties, VisualProperties::default());
    assert!(value.material.is_none());
    assert!(value.plugins.is_empty());
    let loaded = load_visual_str(&value.to_xml().unwrap(), "roundtrip.xml").unwrap();
    assert_eq!(
        loaded.source,
        Source {
            uri: "roundtrip.xml".into(),
            line: 1,
            column: 1
        }
    );
    assert_eq!(loaded.name, value.name);
    assert_eq!(loaded.geometry, value.geometry);
    assert_eq!(loaded.pose.transform, value.pose.transform);
    assert_eq!(loaded.pose.relative_to, value.pose.relative_to);
    assert_eq!(loaded.properties, value.properties);
    assert!(loaded.material.is_none());
    assert!(loaded.plugins.is_empty());
}
pub fn collision(xml: &str, source: &str) {
    let sdf = load_str(xml, source, &NoResources).unwrap();
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!(model.name, "m");
    assert_eq!(model.links.len(), 1);
    let link = &model.links[0];
    assert_eq!(link.name, "l");
    assert_eq!(link.collisions.len(), 1);
    let collision = &link.collisions[0];
    assert_eq!(collision.name, "c");
    assert_eq!(
        collision.source,
        Source {
            uri: source.into(),
            line: 1,
            column: (xml.find("<collision").unwrap() + 1) as u32
        }
    );
    assert_eq!(
        collision.pose.transform,
        Transform {
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.]
        }
    );
    assert!(collision.pose.relative_to.is_empty());
    assert_eq!(collision.surface, Surface::default());
    empty(&collision.geometry);
    assert_eq!(
        sdf.pose_of("m::l", "m").unwrap(),
        Transform {
            translation: [0.; 3],
            rotation: [0., 0., 0., 1.]
        }
    );
    sdf.validate_non_frame_poses().unwrap();
}
