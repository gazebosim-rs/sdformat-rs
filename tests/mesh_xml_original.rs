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
use sdformat::*;
#[path = "support/quiet.rs"]
mod quiet;
fn check(decomposition: bool) {
    let (xml, expected) = if decomposition {
        (
            "<mesh optimization='convex_decomposition'/>",
            MeshOptimization::ConvexDecomposition,
        )
    } else {
        (
            "<mesh optimization='convex_hull'/>",
            MeshOptimization::ConvexHull,
        )
    };
    let mut mesh = load_mesh_str(xml, "constructed.xml").unwrap();
    assert_eq!(mesh.optimization, expected);
    mesh.uri = "mesh-uri".into();
    mesh.scale = [1., 2., 3.];
    mesh.submesh = "submesh".into();
    mesh.center_submesh = false;
    if decomposition {
        mesh.convex_decomposition = Some(ConvexDecomposition {
            max_convex_hulls: 10,
            voxel_resolution: 300000,
        });
    }
    let xml = mesh.to_xml().unwrap();
    assert!(!xml.is_empty());
    let loaded = load_mesh_str(&xml, "mesh-original.xml").unwrap();
    assert_eq!(mesh.optimization, loaded.optimization);
    let optimization = |v| match v {
        MeshOptimization::None => "",
        MeshOptimization::ConvexHull => "convex_hull",
        MeshOptimization::ConvexDecomposition => "convex_decomposition",
    };
    assert_eq!(
        optimization(mesh.optimization),
        optimization(loaded.optimization)
    );
    assert_eq!(mesh.uri, loaded.uri);
    for (a, b) in mesh.scale.into_iter().zip(loaded.scale) {
        assert!((a - b).abs() <= 1e-3);
    }
    assert_eq!(mesh.submesh, loaded.submesh);
    assert_eq!(mesh.center_submesh, loaded.center_submesh);
    if decomposition {
        let d = loaded.convex_decomposition.unwrap();
        assert_eq!(d.max_convex_hulls, 10);
        assert_eq!(d.voxel_resolution, 300000);
    } else {
        assert!(loaded.convex_decomposition.is_none());
    }
}
#[test]
fn original_mesh_to_element() {
    check(true);
}
#[test]
fn original_mesh_to_element_error_output() {
    quiet::assert_quiet(
        "original_mesh_to_element_error_output",
        "SDF_MESH_QUIET_CHILD",
        || check(false),
    );
}
