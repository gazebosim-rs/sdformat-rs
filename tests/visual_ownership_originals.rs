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
use sdformat::{
    Geometry, Material, MaterialScript, Mesh, MeshOptimization, Plugin, Pose, ShaderType, Source,
    Transform, Visual, VisualProperties,
};
use std::f64::consts::{FRAC_PI_2, PI};

fn source() -> Source {
    Source {
        uri: "owned-visual".into(),
        line: 7,
        column: 11,
    }
}

fn visual(name: &str) -> Visual {
    Visual {
        name: name.into(),
        pose: Pose::default(),
        geometry: None,
        properties: VisualProperties::default(),
        material: None,
        plugins: Vec::new(),
        source: source(),
    }
}

fn populated_visual() -> Visual {
    let mut value = visual("test_visual");
    value.properties.cast_shadows = false;
    value.properties.transparency = 0.345;
    value.properties.visibility_flags = 2;
    value.pose = Pose {
        transform: Transform {
            translation: [0., -20., 30.],
            rotation: DQuat::from_euler(EulerRot::ZYX, FRAC_PI_2, -PI, FRAC_PI_2).to_array(),
        },
        relative_to: "link".into(),
    };
    value.material = Some(Box::new(Material {
        ambient: [0.1, 0.1, 0.1, 1.],
        diffuse: [0., 0., 0., 1.],
        specular: [0., 0., 0., 1.],
        emissive: [0., 0., 0., 1.],
        shininess: 0.,
        render_order: 0.,
        lighting: true,
        double_sided: false,
        shader: ShaderType::Pixel,
        normal_map: String::new(),
        pbr: None,
        script: None,
        source: source(),
    }));
    value
}

fn assert_original_observations(value: &Visual) {
    assert_eq!(value.name, "test_visual");
    assert!(!value.properties.cast_shadows);
    // The fixed positive transparency uses native EXPECT_FLOAT_EQ's four ULP.
    assert!(
        value
            .properties
            .transparency
            .to_bits()
            .abs_diff(0.345f32.to_bits())
            <= 4
    );
    assert_eq!(value.properties.transparency.to_bits(), 0.345f32.to_bits()); // Supplemental.
    // Native Pose3 compares each position/quaternion component within 1e-3.
    // Quaternion sign is preserved; opposite-sign equivalence is not substituted.
    for (actual, expected) in value
        .pose
        .transform
        .translation
        .into_iter()
        .zip([0., -20., 30.])
    {
        assert!((actual - expected).abs() <= 1e-3);
    }
    for (actual, expected) in value
        .pose
        .transform
        .rotation
        .into_iter()
        .zip([0.5, -0.5, 0.5, -0.5])
    {
        assert!((actual - expected).abs() <= 1e-3);
    }
    assert_eq!(value.pose.relative_to, "link");
    let material = value.material.as_ref().expect("original nonnull material");
    for (actual, expected) in material.ambient.into_iter().zip([0.1f32, 0.1, 0.1, 1.]) {
        assert!((actual - expected).abs() <= 1e-6); // Native Color equality.
        assert_eq!(actual.to_bits(), expected.to_bits()); // Supplemental exact retention.
    }
    assert_eq!(value.properties.visibility_flags, 2);
}

#[test]
fn original_dom_visual_copy_constructor() {
    let original = populated_visual();
    let copied = original.clone();
    assert_original_observations(&original);
    assert_original_observations(&copied);
    assert_eq!(copied.pose, original.pose); // Supplemental exact owned-pose retention.
}

#[test]
fn original_dom_visual_copy_assignment_operator() {
    let original = populated_visual();
    let mut copied = visual("");
    copied.clone_from(&original);
    assert_original_observations(&original);
    assert_original_observations(&copied);
    assert_eq!(copied.pose, original.pose); // Supplemental.
}

#[test]
fn original_dom_visual_move_constructor() {
    let original = populated_visual();
    let moved = original;
    assert_original_observations(&moved);
}

#[test]
fn original_dom_visual_move_assignment_operator() {
    let original = populated_visual();
    let mut moved = visual("");
    let previous = std::mem::replace(&mut moved, original);
    assert!(previous.name.is_empty()); // Supplemental replacement control.
    assert_original_observations(&moved);
}

#[test]
fn original_dom_visual_copy_assignment_after_move() {
    let first = visual("visual1");
    let second = visual("visual2");
    let temporary = first;
    let first = second.clone();
    let second = temporary.clone();
    assert_eq!(first.name, "visual2");
    assert_eq!(second.name, "visual1");
}

#[test]
fn nested_visual_metadata_remains_owned_and_independent() {
    let mut original = populated_visual();
    original.properties.laser_retro = Some(13.);
    original.geometry = Some(Geometry::Mesh(Mesh {
        uri: "mesh.dae".into(),
        scale: [1., 2., 3.],
        submesh: "part".into(),
        center_submesh: false,
        optimization: MeshOptimization::None,
        convex_decomposition: None,
        source: source(),
    }));
    original.material.as_mut().unwrap().script = Some(MaterialScript {
        uris: vec!["material://definition".into()],
        name: "material_name".into(),
        source: source(),
    });
    let raw_plugin =
        "<plugin name='observer' filename='libobserver.so'><entry>raw</entry></plugin>";
    original.plugins.push(Plugin {
        name: "observer".into(),
        filename: "libobserver.so".into(),
        xml: raw_plugin.into(),
    });
    let mut copied = original.clone();
    let mut assigned = visual("");
    assigned.clone_from(&original);
    assert_eq!(assigned.geometry, original.geometry);
    assert_eq!(assigned.material, original.material);
    assert_eq!(assigned.pose, original.pose);
    assert_eq!(assigned.properties, original.properties);
    copied.material.as_mut().unwrap().ambient[0] = 0.8;
    copied
        .material
        .as_mut()
        .unwrap()
        .script
        .as_mut()
        .unwrap()
        .uris[0]
        .push_str("-changed");
    let Some(Geometry::Mesh(mesh)) = &mut copied.geometry else {
        panic!("expected mesh")
    };
    mesh.uri.push_str("-changed");
    copied.plugins[0].xml.push_str("-changed");
    copied.source.uri.push_str("-changed");
    assert_original_observations(&original);
    let Some(Geometry::Mesh(mesh)) = &original.geometry else {
        panic!("expected original mesh")
    };
    assert_eq!(mesh.uri, "mesh.dae");
    assert_eq!(
        original
            .material
            .as_ref()
            .unwrap()
            .script
            .as_ref()
            .unwrap()
            .uris,
        ["material://definition"]
    );
    assert_eq!(original.plugins[0].xml, raw_plugin);
    assert_eq!(original.source, source());
    drop(original);
    assert_original_observations(&assigned);
    assert_eq!(assigned.properties.laser_retro, Some(13.));
    assert_eq!(assigned.plugins[0].xml, raw_plugin);
    assert_eq!(assigned.source, source());
}
