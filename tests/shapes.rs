/*
 * Copyright 2018 Open Source Robotics Foundation
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
// Assertion port of DOMGeometry.Shapes, geometry_dom.cc:45-335.
use sdformat::*;
fn double(actual: f64, expected: f64) {
    let ordered = |v: f64| {
        let bits = v.to_bits();
        if bits >> 63 != 0 {
            !bits
        } else {
            bits | (1 << 63)
        }
    };
    assert!(
        ordered(actual).abs_diff(ordered(expected)) <= 4,
        "{actual} != {expected}"
    );
}
fn vector<const N: usize>(actual: [f64; N], expected: [f64; N]) {
    for (a, b) in actual.into_iter().zip(expected) {
        let tolerance = if N == 2 { 1e-6 } else { 1e-3 };
        assert!((a - b).abs() <= tolerance, "{a} != {b}");
    }
}
#[test]
fn original_geometry_shapes() {
    let sdf = load_str(
        include_str!("fixtures/shapes.sdf"),
        "shapes.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().unwrap();
    let link = model.links.first().unwrap();
    let collision = |name: &str| {
        &link
            .collisions
            .iter()
            .find(|c| c.name == name)
            .unwrap()
            .geometry
    };
    let visual = |name: &str| {
        link.visuals
            .iter()
            .find(|v| v.name == name)
            .unwrap()
            .geometry
            .as_ref()
            .unwrap()
    };
    let Geometry::Box { size } = collision("box_col") else {
        panic!("box collision")
    };
    vector(*size, [3., 4., 5.]);
    let Geometry::Box { size } = visual("box_vis") else {
        panic!("box visual")
    };
    vector(*size, [1., 2., 3.]);
    let Geometry::Capsule { radius, length } = collision("capsule_col") else {
        panic!("capsule collision")
    };
    double(*radius, 0.2);
    double(*length, 0.1);
    let Geometry::Capsule { radius, length } = visual("capsule_vis") else {
        panic!("capsule visual")
    };
    double(*radius, 2.1);
    double(*length, 10.2);
    let Geometry::Cone { radius, length } = collision("cone_col") else {
        panic!("cone collision")
    };
    double(*radius, 0.2);
    double(*length, 0.1);
    let Geometry::Cone { radius, length } = visual("cone_vis") else {
        panic!("cone visual")
    };
    double(*radius, 2.1);
    double(*length, 10.2);
    let Geometry::Cylinder { radius, length } = collision("cylinder_col") else {
        panic!("cylinder collision")
    };
    double(*radius, 0.2);
    double(*length, 0.1);
    let Geometry::Cylinder { radius, length } = visual("cylinder_vis") else {
        panic!("cylinder visual")
    };
    double(*radius, 2.1);
    double(*length, 10.2);
    let Geometry::Ellipsoid { radii } = collision("ellipsoid_col") else {
        panic!("ellipsoid collision")
    };
    vector(*radii, [1., 2., 3.]);
    let Geometry::Ellipsoid { radii } = visual("ellipsoid_vis") else {
        panic!("ellipsoid visual")
    };
    vector(*radii, [0.1, 0.2, 0.3]);
    let Geometry::Plane { normal, size } = collision("plane_col") else {
        panic!("plane collision")
    };
    vector(*normal, [1., 0., 0.]);
    vector(*size, [1.4, 6.3]);
    let Geometry::Plane { normal, size } = visual("plane_vis") else {
        panic!("plane visual")
    };
    vector(*normal, [0., 1., 0.]);
    vector(*size, [2., 4.]);
    let Geometry::Sphere { radius } = collision("sphere_col") else {
        panic!("sphere collision")
    };
    double(*radius, 23.4);
    let Geometry::Sphere { radius } = visual("sphere_vis") else {
        panic!("sphere visual")
    };
    double(*radius, 100.2);
    let Geometry::Mesh(mesh) = collision("mesh_col") else {
        panic!("mesh collision")
    };
    assert_eq!(
        match mesh.optimization {
            MeshOptimization::None => "none",
            MeshOptimization::ConvexHull => "convex_hull",
            MeshOptimization::ConvexDecomposition => "convex_decomposition",
        },
        "convex_decomposition"
    );
    assert_eq!(mesh.optimization, MeshOptimization::ConvexDecomposition);
    let decomposition = mesh.convex_decomposition.as_ref().unwrap();
    assert_eq!(decomposition.max_convex_hulls, 4);
    assert_eq!(decomposition.voxel_resolution, 400000);
    let base = "https://fuel.gazebosim.org/1.0/an_org/models/a_model/";
    assert_eq!(mesh.uri, format!("{base}mesh/mesh.dae"));
    vector(mesh.scale, [0.1, 0.2, 0.3]);
    assert_eq!(mesh.submesh, "my_submesh");
    assert!(mesh.center_submesh);
    let Geometry::Mesh(mesh) = visual("mesh_vis") else {
        panic!("mesh visual")
    };
    assert_eq!(mesh.uri, format!("{base}mesh/mesh.dae"));
    vector(mesh.scale, [1.2, 2.3, 3.4]);
    assert_eq!(mesh.submesh, "another_submesh");
    assert!(!mesh.center_submesh);
    let Geometry::Heightmap(heightmap) = collision("heightmap_col") else {
        panic!("heightmap collision")
    };
    assert_eq!(
        heightmap.uri,
        format!("{base}materials/textures/heightmap.png")
    );
    vector(heightmap.size, [500., 500., 100.]);
    vector(heightmap.position, [1., 2., 3.]);
    assert_eq!(heightmap.textures.len(), 0);
    assert_eq!(heightmap.blends.len(), 0);
    let Geometry::Heightmap(heightmap) = visual("heightmap_vis") else {
        panic!("heightmap visual")
    };
    assert_eq!(
        heightmap.uri,
        format!("{base}materials/textures/heightmap.png")
    );
    vector(heightmap.size, [500., 500., 100.]);
    vector(heightmap.position, [1., 2., 3.]);
    assert_eq!(heightmap.textures.len(), 3);
    assert_eq!(heightmap.blends.len(), 2);
    for (i, size) in [5., 10., 20.].into_iter().enumerate() {
        let texture = &heightmap.textures[i];
        assert_eq!(
            texture.diffuse,
            format!("{base}materials/textures/diffuse{i}.png")
        );
        assert_eq!(
            texture.normal,
            format!("{base}materials/textures/normal{i}.png")
        );
        double(texture.size, size);
    }
    double(heightmap.blends[0].min_height, 15.);
    double(heightmap.blends[0].fade_distance, 5.);
    double(heightmap.blends[1].min_height, 30.);
    double(heightmap.blends[1].fade_distance, 10.);
    let Geometry::Polyline(lines) = collision("polyline_col") else {
        panic!("polyline collision")
    };
    assert!(!lines.is_empty());
    assert_eq!(lines.len(), 2);
    double(lines[0].height, 0.5);
    assert_eq!(lines[0].points.len(), 5);
    vector(lines[0].points[0], [-0.5, -0.5]);
    vector(lines[0].points[1], [-0.5, 0.5]);
    double(lines[1].height, 0.3);
    assert_eq!(lines[1].points.len(), 4);
    vector(lines[1].points[0], [-0.3, -0.3]);
    vector(lines[1].points[1], [-0.3, 0.3]);
    let Geometry::Polyline(lines) = visual("polyline_vis") else {
        panic!("polyline visual")
    };
    assert!(!lines.is_empty());
    assert_eq!(lines.len(), 1);
    double(lines[0].height, 1.);
    assert_eq!(lines[0].points.len(), 3);
    vector(lines[0].points[0], [-0.2, -0.2]);
    vector(lines[0].points[1], [-0.2, 0.2]);
}
