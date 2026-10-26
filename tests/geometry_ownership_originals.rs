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
    ConvexDecomposition, Geometry, Heightmap, HeightmapBlend, HeightmapTexture, Mesh,
    MeshOptimization, Polyline, Source,
};

fn box_size(geometry: &Geometry) -> [f64; 3] {
    let Geometry::Box { size } = geometry else {
        panic!("expected box metadata")
    };
    *size
}

fn assert_size(geometry: &Geometry, expected: [f64; 3]) {
    for (actual, expected) in box_size(geometry).into_iter().zip(expected) {
        // The exact installed native Vector3 operator== uses this component bound.
        assert!((actual - expected).abs() <= 1e-3);
        assert_eq!(actual.to_bits(), expected.to_bits()); // Supplemental exact retention.
    }
}

#[test]
fn original_dom_box_move_constructor_size_subset() {
    let original = Geometry::Box { size: [1., 2., 3.] };
    let moved = original;
    assert_size(&moved, [1., 2., 3.]);
    // Native Shape().Volume() and Shape().Size() remain unported; this case is partial.
}

#[test]
fn original_dom_box_copy_constructor() {
    let original = Geometry::Box {
        size: [0.1, 0.2, 0.3],
    };
    let copied = original.clone();
    assert_size(&copied, [0.1, 0.2, 0.3]);
}

#[test]
fn original_dom_box_copy_assigment_operator() {
    let original = Geometry::Box {
        size: [0.2, 0.3, 0.4],
    };
    let mut copied = Geometry::Box { size: [1.; 3] };
    copied.clone_from(&original);
    assert_size(&copied, [0.2, 0.3, 0.4]);
}

#[test]
fn original_dom_box_move_assignment_constructor() {
    let original = Geometry::Box { size: [1., 2., 3.] };
    let mut moved = Geometry::Box { size: [1.; 3] };
    let previous = std::mem::replace(&mut moved, original);
    assert_size(&previous, [1.; 3]); // Supplemental replacement control.
    assert_size(&moved, [1., 2., 3.]);
}

#[test]
fn original_dom_box_copy_assignment_after_move() {
    let first = Geometry::Box { size: [1., 2., 3.] };
    let second = Geometry::Box { size: [4., 5., 6.] };
    let tmp = first;
    let first = second.clone();
    let second = tmp.clone();
    assert_size(&first, [4., 5., 6.]);
    assert_size(&second, [1., 2., 3.]);
}

fn source() -> Source {
    Source {
        uri: "owned-geometry".into(),
        line: 7,
        column: 11,
    }
}

#[test]
fn owned_geometry_payloads_remain_independent_after_cloning() {
    let mut geometries = vec![
        Geometry::Box { size: [1., 2., 3.] },
        Geometry::Sphere { radius: 0.5 },
        Geometry::Cylinder {
            radius: 0.5,
            length: 2.,
        },
        Geometry::Capsule {
            radius: 0.5,
            length: 2.,
        },
        Geometry::Cone {
            radius: 0.5,
            length: 2.,
        },
        Geometry::Ellipsoid {
            radii: [1., 2., 3.],
        },
        Geometry::Plane {
            normal: [0., 0., 1.],
            size: [2., 3.],
        },
        Geometry::Heightmap(Box::new(Heightmap {
            uri: "height.png".into(),
            size: [1., 2., 3.],
            position: [4., 5., 6.],
            use_terrain_paging: true,
            sampling: 3,
            textures: vec![HeightmapTexture {
                size: 2.,
                diffuse: "diffuse".into(),
                normal: "normal".into(),
            }],
            blends: vec![HeightmapBlend {
                min_height: 1.,
                fade_distance: 2.,
            }],
            source: source(),
        })),
        Geometry::Polyline(vec![Polyline {
            height: 2.,
            points: vec![[1., 2.], [3., 4.]],
            source: source(),
        }]),
    ];
    for optimization in [
        MeshOptimization::None,
        MeshOptimization::ConvexHull,
        MeshOptimization::ConvexDecomposition,
    ] {
        geometries.push(Geometry::Mesh(Mesh {
            uri: "mesh.dae".into(),
            scale: [1., 2., 3.],
            submesh: "part".into(),
            center_submesh: true,
            optimization,
            convex_decomposition: (optimization == MeshOptimization::ConvexDecomposition)
                .then_some(ConvexDecomposition {
                    max_convex_hulls: 4,
                    voxel_resolution: 1000,
                }),
            source: source(),
        }));
    }
    for original in geometries {
        let copied = original.clone();
        let mut assigned = Geometry::Box { size: [1.; 3] };
        assigned.clone_from(&original);
        assert_eq!(copied, original);
        assert_eq!(assigned, original);
        drop(original);
        match (&mut assigned, &copied) {
            (Geometry::Mesh(changed), Geometry::Mesh(retained)) => {
                changed.uri.push_str("-changed");
                changed.source.uri.push_str("-changed");
                assert_eq!(retained.uri, "mesh.dae");
                assert_eq!(retained.source, source());
                if let Some(decomposition) = &mut changed.convex_decomposition {
                    decomposition.max_convex_hulls = 99;
                    assert_eq!(retained.convex_decomposition.unwrap().max_convex_hulls, 4);
                }
            }
            (Geometry::Heightmap(changed), Geometry::Heightmap(retained)) => {
                changed.textures[0].diffuse.push_str("-changed");
                changed.blends[0].fade_distance = 99.;
                changed.source.uri.push_str("-changed");
                assert_eq!(retained.textures[0].diffuse, "diffuse");
                assert_eq!(retained.blends[0].fade_distance, 2.);
                assert_eq!(retained.source, source());
            }
            (Geometry::Polyline(changed), Geometry::Polyline(retained)) => {
                changed[0].points[0][0] = 99.;
                changed[0].source.uri.push_str("-changed");
                assert_eq!(retained[0].points, [[1., 2.], [3., 4.]]);
                assert_eq!(retained[0].source, source());
            }
            _ => {}
        }
    }
}
