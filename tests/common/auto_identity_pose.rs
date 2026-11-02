/*
 * Copyright (C) 2018 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

/// Exact two known native identity-pose inputs, with their complete values.
pub fn assert_identity_pose(row: &serde_json::Value, uri: &str, is_sphere: bool) {
    let xml = row["xml"].as_str().unwrap();
    assert_eq!(row["rust_error"], "Unsupported");
    assert!(xml.contains("<pose>0 0 0 0 0 0</pose>"));
    let lines: Vec<_> = row["stdout"].as_str().unwrap().lines().collect();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0], "LOAD");
    assert_eq!(lines[2], "RESOLVE");
    assert_eq!(lines[1], lines[3]);
    let native: Vec<u64> = lines[1]
        .strip_prefix("VALUES ")
        .unwrap()
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect();
    let (mass, diagonal) = if is_sphere {
        (4621306650028725639, 4585400661777136886)
    } else {
        (4666679192002232320, 4663993451599451477)
    };
    assert_eq!(native.len(), 18);
    assert_eq!(&native[..2], [1, 1]);
    assert_eq!(native[2], mass);
    assert_eq!(&native[3..6], [diagonal; 3]);
    assert_eq!(
        &native[6..],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 4607182418800017408, 0, 0]
    );
    if is_sphere {
        crate::sphere::assert_complete_sphere(
            xml,
            uri,
            "1.12",
            "sphere_col",
            (2170., 0.1, 4621306650028725639, 4585400661777136886),
        );
    } else {
        assert_box(xml, uri);
    }
}

fn assert_box(xml: &str, uri: &str) {
    let sdf = load_str(xml, uri, &NoResources).unwrap();
    assert_eq!(sdf.version, "1.12");
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!(model.name, "shapes");
    assert_eq!(model.scoped_name, "shapes");
    assert_eq!(model.pose, Pose::default());
    assert!(!(model.is_static || model.self_collide || model.enable_wind));
    assert!(model.allow_auto_disable);
    assert!(model.raw_namespace.is_none());
    assert!(model.canonical_link.is_empty());
    assert!(model.placement_frame.is_empty());
    assert!(model.include.is_none());
    assert!(model.merged_includes.is_empty());
    assert!(model.models.is_empty());
    assert!(model.frames.is_empty());
    assert!(model.joints.is_empty());
    assert!(model.plugins.is_empty());
    assert!(model.extensions.is_empty());
    let children = model.element().unwrap().children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].tag(), "link");
    assert_eq!(children[0].name_attribute(), Some("link"));
    assert_eq!(model.links.len(), 1);
    let link = &model.links[0];
    assert_eq!(link.name, "link");
    assert_eq!(link.scoped_name, "shapes::link");
    assert_eq!(link.pose, Pose::default());
    assert_eq!(link.properties, LinkProperties::default());
    assert!(link.velocity_decay.is_none());
    assert!(link.visuals.is_empty());
    assert!(link.sensors.is_empty());
    assert!(link.lights.is_empty());
    assert!(link.particle_emitters.is_empty());
    assert!(link.projectors.is_empty());
    assert!(link.plugins.is_empty());
    assert_eq!(link.collisions.len(), 1);
    let collision = &link.collisions[0];
    assert_eq!(collision.name, "box_col");
    assert_eq!(collision.density, Some(1240.));
    assert_eq!(collision.density(), 1240.);
    assert_eq!(collision.pose, Pose::default());
    assert!(matches!(
        collision.geometry,
        Geometry::Box { size: [2., 2., 2.] }
    ));
    assert_eq!(collision.surface, Surface::default());
    for (source, tag) in [
        (&model.source, "model"),
        (&link.source, "link"),
        (&collision.source, "collision"),
    ] {
        assert_eq!(source.uri, uri);
        assert_eq!(source.line, 1);
        assert_eq!(
            source.column,
            u32::try_from(xml.find(&format!("<{tag}")).unwrap() + 1).unwrap()
        );
    }
    // This original box is exact even through Parry's reciprocal representation.
    assert_eq!(link.inertial.mass.to_bits(), 4666679192002232320);
    assert_eq!(
        link.inertial.inertia.map(f64::to_bits),
        [
            4663993451599451477,
            4663993451599451477,
            4663993451599451477,
            0,
            0,
            0
        ]
    );
    assert_eq!(link.inertial.pose, Pose::default());
    assert!(link.inertial.fluid_added_mass.is_none());
    assert_eq!(
        sdf.pose_of("shapes::link", "::").unwrap(),
        Transform::default()
    );
    sdf.validate_non_frame_poses().unwrap();
}
