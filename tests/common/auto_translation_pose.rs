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

pub fn assert_unit_translation(xml: &str, uri: &str, sphere: bool) {
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
    assert_eq!(
        collision.name,
        if sphere { "sphere_col" } else { "box_col" }
    );
    let density = if sphere { 2170. } else { 1240. };
    assert_eq!(collision.density, Some(density));
    assert_eq!(collision.density(), density);
    assert_eq!(collision.pose.transform.translation, [1., 0., 0.]);
    assert_eq!(
        collision.pose.transform.rotation,
        Transform::default().rotation
    );
    assert!(collision.pose.relative_to.is_empty());
    assert_eq!(
        collision.geometry,
        if sphere {
            Geometry::Sphere { radius: 0.1 }
        } else {
            Geometry::Box { size: [2.; 3] }
        }
    );
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
    let (mass, diagonal) = if sphere {
        (4621306650028725639, 4585400661777136886)
    } else {
        (4666679192002232320, 4663993451599451477)
    };
    assert_eq!(link.inertial.mass.to_bits(), mass);
    assert_eq!(
        link.inertial.inertia.map(f64::to_bits),
        [diagonal, diagonal, diagonal, 0, 0, 0]
    );
    assert_eq!(link.inertial.pose.transform.translation, [1., 0., 0.]);
    assert_eq!(
        link.inertial.pose.transform.rotation,
        Transform::default().rotation
    );
    assert!(link.inertial.pose.relative_to.is_empty());
    assert!(link.inertial.fluid_added_mass.is_none());
    assert_eq!(
        sdf.pose_of("shapes::link", "::").unwrap(),
        Transform::default()
    );
    assert_eq!(
        sdf.resolve_pose(&link.inertial.pose, "shapes", "shapes::link", "::")
            .unwrap()
            .translation,
        [1., 0., 0.]
    );
    sdf.validate_non_frame_poses().unwrap();
    let mut copied = model.clone();
    assert_eq!(copied.element(), model.element());
    copied.links[0].collisions[0].pose.transform.translation = [2., 0., 0.];
    copied.links[0].inertial.pose.transform.translation = [2., 0., 0.];
    assert_eq!(collision.pose.transform.translation, [1., 0., 0.]);
    assert_eq!(link.inertial.pose.transform.translation, [1., 0., 0.]);
}
