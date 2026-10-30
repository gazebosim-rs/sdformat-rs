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
 *
*/

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use glam::{DQuat, EulerRot};
use sdformat::{Collision, Geometry, Mesh, MeshOptimization, Pose, Source, Surface, Transform};
use std::f64::consts::PI;

fn source() -> Source {
    Source {
        uri: "owned-collision".into(),
        line: 7,
        column: 11,
    }
}

fn default_collision() -> Collision {
    Collision {
        density: None,
        name: String::new(),
        pose: Pose::default(),
        geometry: Geometry::Empty,
        surface: Surface::default(),
        // Rust requires a source value; native unattached Element identity is not mapped.
        source: source(),
    }
}

fn collision(position: [f64; 3]) -> Collision {
    let mut value = default_collision();
    value.pose.transform = Transform {
        translation: position,
        rotation: DQuat::from_euler(EulerRot::ZYX, PI, PI, PI).to_array(),
    };
    value
}

fn assert_original_pose(value: &Collision, expected: [f64; 3]) {
    // Native Pose3 equality uses component-wise <=1e-3 with no sign relaxation.
    for (actual, expected) in value.pose.transform.translation.into_iter().zip(expected) {
        assert!((actual - expected).abs() <= 1e-3);
    }
    for (actual, expected) in value
        .pose
        .transform
        .rotation
        .into_iter()
        .zip([0., 0., 0., 1.])
    {
        assert!((actual - expected).abs() <= 1e-3);
    }
}

#[test]
fn original_dom_collision_move_constructor() {
    let original = collision([-10., -20., -30.]);
    let moved = original;
    assert_eq!(moved.geometry, Geometry::Empty);
    assert_original_pose(&moved, [-10., -20., -30.]);
}

#[test]
fn original_dom_collision_copy_constructor() {
    let original = collision([-10., -20., -30.]);
    let copied = original.clone();
    assert_eq!(copied.geometry, Geometry::Empty);
    assert_original_pose(&copied, [-10., -20., -30.]);
    assert_eq!(copied.pose, original.pose); // Supplemental exact retention.
}

#[test]
fn original_dom_collision_move_assignment() {
    let original = collision([-10., -20., -30.]);
    let mut moved = default_collision();
    let previous = std::mem::replace(&mut moved, original);
    assert_eq!(moved.geometry, Geometry::Empty);
    assert_original_pose(&moved, [-10., -20., -30.]);
    assert_eq!(previous.pose.transform.translation, [0.; 3]); // Supplemental.
}

#[test]
fn original_dom_collision_copy_assignment() {
    let original = collision([-10., -20., -30.]);
    let mut copied = default_collision();
    copied.clone_from(&original);
    assert_eq!(copied.geometry, Geometry::Empty);
    assert_original_pose(&copied, [-10., -20., -30.]);
    assert_eq!(copied.pose, original.pose); // Supplemental.
}

#[test]
fn original_dom_collision_copy_assignment_after_move() {
    let first = collision([-10., -20., -30.]);
    let second = collision([-20., -30., -40.]);
    let temporary = first;
    let first = second.clone();
    let second = temporary.clone();
    assert_eq!(first.geometry, Geometry::Empty);
    assert_eq!(second.geometry, Geometry::Empty);
    assert_original_pose(&first, [-20., -30., -40.]);
    assert_original_pose(&second, [-10., -20., -30.]);
}

#[test]
fn nested_collision_metadata_clones_remain_owned_and_independent() {
    let mut original = collision([-10., -20., -30.]);
    original.geometry = Geometry::Mesh(Mesh {
        uri: "mesh.dae".into(),
        scale: [1., 2., 3.],
        submesh: "part".into(),
        center_submesh: false,
        optimization: MeshOptimization::None,
        convex_decomposition: None,
        source: source(),
    });
    original.surface.source = Some(source());
    original.surface.contact.source = Some(source());
    original.surface.contact.category_bitmask = Some(0x1234);
    original.surface.friction.ode.source = Some(source());
    let mut copied = original.clone();
    let mut assigned = collision([0.; 3]);
    assigned.clone_from(&original);
    assert_eq!(assigned.geometry, original.geometry);
    assert_eq!(assigned.surface, original.surface);
    assert_eq!(assigned.pose, original.pose);
    let Geometry::Mesh(mesh) = &mut copied.geometry else {
        panic!("expected mesh")
    };
    mesh.uri.push_str("-changed");
    copied
        .surface
        .source
        .as_mut()
        .unwrap()
        .uri
        .push_str("-changed");
    copied
        .surface
        .contact
        .source
        .as_mut()
        .unwrap()
        .uri
        .push_str("-changed");
    copied.surface.friction.ode.mu = 7.;
    copied.source.uri.push_str("-changed");
    let Geometry::Mesh(mesh) = &original.geometry else {
        panic!("expected original mesh")
    };
    assert_eq!(mesh.uri, "mesh.dae");
    assert_eq!(original.surface.source, Some(source()));
    assert_eq!(original.surface.contact.source, Some(source()));
    assert_eq!(original.surface.friction.ode.mu, 1.);
    assert_eq!(original.source, source());
    drop(original);
    assert_original_pose(&assigned, [-10., -20., -30.]);
    assert_eq!(assigned.surface.contact.category_bitmask, Some(0x1234));
    assert_eq!(assigned.surface.source, Some(source()));
    assert_eq!(assigned.source, source());
}

#[test]
fn replacing_a_preconfigured_box_destination_remains_supported() {
    let mut target = collision([0.; 3]);
    target.name = "collision".into();
    target.geometry = Geometry::Box { size: [1., 2., 3.] };
    let previous = std::mem::replace(&mut target, collision([-10., -20., -30.]));
    assert_eq!(previous.name, "collision");
    assert_eq!(previous.geometry, Geometry::Box { size: [1., 2., 3.] });
    assert_eq!(previous.pose.transform.translation, [0.; 3]);
    assert_original_pose(&target, [-10., -20., -30.]);

    let mut copied = previous;
    copied.clone_from(&target);
    assert_original_pose(&copied, [-10., -20., -30.]);
    assert_eq!(copied.pose, target.pose);
}
