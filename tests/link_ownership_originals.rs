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
use sdformat::{
    Collision, Geometry, Inertial, Link, LinkProperties, Mesh, MeshOptimization, Plugin, Pose,
    Sensor, Source, Surface, Visual, VisualProperties,
};

fn source() -> Source {
    Source {
        uri: "owned-link".into(),
        line: 7,
        column: 11,
    }
}

fn link(name: &str) -> Link {
    Link {
        name: name.into(),
        scoped_name: format!("model::{name}"),
        pose: Pose::default(),
        inertial: Inertial::default(),
        collisions: Vec::new(),
        visuals: Vec::new(),
        lights: Vec::new(),
        particle_emitters: Vec::new(),
        projectors: Vec::new(),
        sensors: Vec::new(),
        properties: LinkProperties::default(),
        plugins: Vec::new(),
        source: source(),
    }
}

#[test]
fn original_dom_link_copy_constructor() {
    let original = link("test_link");
    let copied = original.clone();
    assert_eq!(copied.name, "test_link");
}

#[test]
fn original_dom_link_copy_assignment_operator() {
    let original = link("test_link");
    let mut copied = link("");
    copied.clone_from(&original);
    assert_eq!(copied.name, "test_link");
}

#[test]
fn original_dom_link_move_constructor() {
    let original = link("test_link");
    // The native case named MoveConstructor actually invokes link2(link), a copy.
    let copied = original.clone();
    assert_eq!(copied.name, "test_link");
}

#[test]
fn original_dom_link_move_assignment_operator() {
    let original = link("test_link");
    let mut moved = link("");
    let previous = std::mem::replace(&mut moved, original);
    assert_eq!(moved.name, "test_link");
    assert!(previous.name.is_empty()); // Supplemental replacement control.
}

#[test]
fn original_dom_link_copy_assignment_after_move() {
    let first = link("link1");
    let second = link("link2");
    let temporary = first;
    let first = second.clone();
    let second = temporary.clone();
    assert_eq!(first.name, "link2");
    assert_eq!(second.name, "link1");
}

#[test]
fn nested_link_and_inertial_metadata_remain_owned_and_independent() {
    let mut original = link("test_link");
    original.pose.relative_to = "body-frame".into();
    original.inertial.mass = 2.;
    original.inertial.pose.relative_to = "mass-frame".into();
    original.inertial.fluid_added_mass = Some(Box::new([[0.; 6]; 6]));
    original.inertial.fluid_added_mass.as_mut().unwrap()[0][0] = 3.;
    original.properties.enable_wind = true;
    let mut surface = Surface::default();
    surface.contact.source = Some(source());
    original.collisions.push(Collision {
        name: "collision".into(),
        pose: Pose::default(),
        geometry: Geometry::Mesh(Mesh {
            uri: "mesh.dae".into(),
            scale: [1., 2., 3.],
            submesh: "part".into(),
            center_submesh: false,
            optimization: MeshOptimization::None,
            convex_decomposition: None,
            source: source(),
        }),
        surface,
        source: source(),
    });
    original.visuals.push(Visual {
        name: "visual".into(),
        pose: Pose::default(),
        geometry: Some(Geometry::Box { size: [1., 2., 3.] }),
        properties: VisualProperties::default(),
        material: None,
        plugins: Vec::new(),
        source: source(),
    });
    original.sensors.push(Sensor {
        name: "sensor".into(),
        topic: "sensor-topic".into(),
        source: source(),
        ..Sensor::default()
    });
    let raw = "<plugin name='p' filename='libp.so'><entry>raw</entry></plugin>";
    original.plugins.push(Plugin {
        name: "p".into(),
        filename: "libp.so".into(),
        xml: raw.into(),
    });
    let mut copied = original.clone();
    let mut assigned = link("");
    assigned.clone_from(&original);
    assert_eq!(
        assigned.inertial.fluid_added_mass,
        original.inertial.fluid_added_mass
    );
    assert_eq!(
        assigned.collisions[0].geometry,
        original.collisions[0].geometry
    );
    assert_eq!(
        assigned.collisions[0].surface,
        original.collisions[0].surface
    );
    assert_eq!(assigned.visuals[0].geometry, original.visuals[0].geometry);
    assert_eq!(assigned.properties, original.properties);
    copied.inertial.mass = 9.;
    copied.inertial.fluid_added_mass.as_mut().unwrap()[0][0] = 7.;
    copied.inertial.pose.relative_to.push_str("-changed");
    copied.pose.relative_to.push_str("-changed");
    let Geometry::Mesh(mesh) = &mut copied.collisions[0].geometry else {
        panic!("expected mesh")
    };
    mesh.uri.push_str("-changed");
    copied.collisions[0]
        .surface
        .contact
        .source
        .as_mut()
        .unwrap()
        .uri
        .push_str("-changed");
    copied.visuals[0].name.push_str("-changed");
    copied.sensors[0].topic.push_str("-changed");
    copied.plugins[0].xml.push_str("-changed");
    copied.scoped_name.push_str("-changed");
    copied.source.uri.push_str("-changed");
    assert_eq!(original.inertial.mass, 2.);
    assert_eq!(
        original.inertial.fluid_added_mass.as_ref().unwrap()[0][0],
        3.
    );
    assert_eq!(original.inertial.pose.relative_to, "mass-frame");
    assert_eq!(original.pose.relative_to, "body-frame");
    let Geometry::Mesh(mesh) = &original.collisions[0].geometry else {
        panic!("expected original mesh")
    };
    assert_eq!(mesh.uri, "mesh.dae");
    assert_eq!(
        original.collisions[0].surface.contact.source,
        Some(source())
    );
    assert_eq!(original.visuals[0].name, "visual");
    assert_eq!(original.sensors[0].topic, "sensor-topic");
    assert_eq!(original.plugins[0].xml, raw);
    assert_eq!(original.scoped_name, "model::test_link");
    assert_eq!(original.source, source());
    drop(original);
    assert_eq!(assigned.name, "test_link");
    assert_eq!(assigned.inertial.mass, 2.);
    assert_eq!(
        assigned.inertial.fluid_added_mass.as_ref().unwrap()[0][0],
        3.
    );
    assert_eq!(assigned.inertial.pose.relative_to, "mass-frame");
    assert_eq!(assigned.visuals[0].name, "visual");
    assert_eq!(assigned.sensors[0].topic, "sensor-topic");
    assert_eq!(assigned.plugins[0].xml, raw);
    assert_eq!(assigned.source, source());
}
