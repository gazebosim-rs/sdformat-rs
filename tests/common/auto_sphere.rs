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

pub fn assert_complete_sphere(
    xml: &str,
    uri: &str,
    version: &str,
    collision_name: &str,
    expected: (f64, f64, u64, u64),
) {
    let (density, radius, mass, diagonal) = expected;
    let sdf = load_str(xml, uri, &NoResources).unwrap();
    assert_eq!(sdf.version, version);
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
    assert_eq!(collision.name, collision_name);
    assert_eq!(collision.density, Some(density));
    assert_eq!(collision.density(), density);
    assert_eq!(collision.pose, Pose::default());
    assert_eq!(collision.geometry, Geometry::Sphere { radius });
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
    assert_eq!(link.inertial.mass.to_bits(), mass);
    assert_eq!(
        link.inertial.inertia.map(f64::to_bits),
        [diagonal, diagonal, diagonal, 0, 0, 0]
    );
    assert_eq!(link.inertial.pose, Pose::default());
    assert!(link.inertial.fluid_added_mass.is_none());
    assert_eq!(
        sdf.pose_of("shapes::link", "::").unwrap(),
        Transform::default()
    );
    sdf.validate_non_frame_poses().unwrap();
    let mut cloned = model.clone();
    assert_eq!(cloned.element(), model.element());
    cloned.links[0].inertial.mass += 1.;
    assert_eq!(link.inertial.mass.to_bits(), mass);
}
