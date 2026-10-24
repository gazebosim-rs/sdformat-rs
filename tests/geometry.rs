// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
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
use sdformat::*;

fn observed(geometry: &Geometry) -> serde_json::Value {
    match geometry {
        Geometry::Capsule { radius, length } => {
            serde_json::json!({"kind":"capsule", "radius":radius, "length":length})
        }
        Geometry::Cone { radius, length } => {
            serde_json::json!({"kind":"cone", "radius":radius, "length":length})
        }
        Geometry::Ellipsoid { radii } => serde_json::json!({"kind":"ellipsoid", "radii":radii}),
        Geometry::Mesh(mesh) => serde_json::json!({
            "kind":"mesh", "uri":mesh.uri, "scale":mesh.scale, "submesh":mesh.submesh,
            "center_submesh":mesh.center_submesh,
            "optimization":match mesh.optimization { MeshOptimization::None => "", MeshOptimization::ConvexHull => "convex_hull", MeshOptimization::ConvexDecomposition => "convex_decomposition" },
            "convex_decomposition":mesh.convex_decomposition.map(|d| [d.max_convex_hulls,d.voxel_resolution])
        }),
        _ => panic!("unexpected geometry"),
    }
}
#[test]
fn native_geometry_metadata_and_default_corpus() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("geometry.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let result = load_str(
            case["xml"].as_str().unwrap(),
            case["id"].as_str().unwrap(),
            &NoResources,
        );
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{case}"
        );
        if let Some(expected) = case.get("link_mass") {
            assert_eq!(
                result.as_ref().unwrap().models[0].links[0].inertial.mass,
                expected.as_f64().unwrap(),
                "{case}"
            );
        }
        if let Some(expected) = case.get("geometry") {
            let sdf = result.unwrap();
            assert_eq!(
                observed(&sdf.models[0].links[0].collisions[0].geometry),
                *expected,
                "{case}"
            );
        }
    }
}
#[test]
fn visual_mesh_replaces_the_previous_explicit_unsupported_boundary() {
    let xml = "<sdf version='1.12'><model name='m'><link name='l'><visual name='v'><geometry><mesh><uri>mesh.dae</uri></mesh></geometry></visual></link></model></sdf>";
    let sdf = load_str(xml, "visuals", &NoResources).unwrap();
    let Geometry::Mesh(mesh) = sdf.models[0].links[0].visuals[0].geometry.as_ref().unwrap() else {
        panic!("expected mesh")
    };
    assert_eq!(mesh.uri, "mesh.dae");
    assert_eq!(mesh.source.uri, "visuals");
}

// These ports preserve the typed-shape presence and value assertions from
// src/Geometry_TEST.cc. XML construction replaces the C++ mutable setters.
fn shape(xml: &str) -> Geometry {
    let sdf=load_str(&format!("<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><geometry>{xml}</geometry></collision></link></model></sdf>"),"geometry-ports",&NoResources).unwrap();
    sdf.models
        .into_iter()
        .next()
        .unwrap()
        .links
        .into_iter()
        .next()
        .unwrap()
        .collisions
        .into_iter()
        .next()
        .unwrap()
        .geometry
}
fn original_vector<const N: usize>(actual: [f64; N], expected: [f64; N]) {
    for (a, b) in actual.into_iter().zip(expected) {
        assert!((a - b).abs() <= 1e-3);
    }
    assert_eq!(actual, expected); // Supplementary exact check.
}
#[test]
fn geometry_box_original_assertions() {
    let Geometry::Box { size } = shape("<box><size>1 2 3</size></box>") else {
        panic!("expected BOX and non-null BoxShape")
    };
    original_vector(size, [1., 2., 3.]);
}
#[test]
fn geometry_sphere_original_assertions() {
    let Geometry::Sphere { radius } = shape("<sphere><radius>0.123</radius></sphere>") else {
        panic!("expected SPHERE and non-null SphereShape")
    };
    assert_eq!(radius, 0.123); // Stronger than original EXPECT_DOUBLE_EQ.
}
#[test]
fn geometry_capsule_original_assertions() {
    let Geometry::Capsule { radius, length } =
        shape("<capsule><radius>0.123</radius><length>4.56</length></capsule>")
    else {
        panic!("expected CAPSULE and non-null CapsuleShape")
    };
    assert_eq!(radius, 0.123);
    assert_eq!(length, 4.56);
}
#[test]
fn geometry_cone_original_assertions() {
    let Geometry::Cone { radius, length } =
        shape("<cone><radius>0.123</radius><length>4.56</length></cone>")
    else {
        panic!("expected CONE and non-null ConeShape")
    };
    assert_eq!(radius, 0.123);
    assert_eq!(length, 4.56);
}
#[test]
fn geometry_cylinder_original_assertions() {
    let Geometry::Cylinder { radius, length } =
        shape("<cylinder><radius>0.123</radius><length>4.56</length></cylinder>")
    else {
        panic!("expected CYLINDER and non-null CylinderShape")
    };
    assert_eq!(radius, 0.123);
    assert_eq!(length, 4.56);
}
#[test]
fn geometry_ellipsoid_original_assertions() {
    let Geometry::Ellipsoid { radii } = shape("<ellipsoid><radii>1 2 3</radii></ellipsoid>") else {
        panic!("expected ELLIPSOID and non-null EllipsoidShape")
    };
    original_vector(radii, [1., 2., 3.]);
}
#[test]
fn geometry_mesh_original_assertions() {
    let Geometry::Mesh(mesh) = shape(
        "<mesh><uri>banana</uri><scale>1 2 3</scale><submesh><name>orange</name><center>true</center></submesh></mesh>",
    ) else {
        panic!("expected MESH and non-null MeshShape")
    };
    original_vector(mesh.scale, [1., 2., 3.]);
    assert_eq!(mesh.uri, "banana");
    assert_eq!(mesh.submesh, "orange");
    assert!(mesh.center_submesh);
}
#[test]
fn geometry_plane_original_assertions() {
    let Geometry::Plane { normal, size } =
        shape("<plane><normal>1 0 0</normal><size>9 8</size></plane>")
    else {
        panic!("expected PLANE and non-null PlaneShape")
    };
    original_vector(normal, [1., 0., 0.]);
    original_vector(size, [9., 8.]);
}
