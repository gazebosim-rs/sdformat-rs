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
use sdformat::{NoResources, ShaderType, load_str};
fn color(actual: [f32; 4], expected: [f32; 4]) {
    for (a, b) in actual.into_iter().zip(expected) {
        assert!((a - b).abs() <= 1e-6_f32, "{a} != {b}");
    }
}
#[test]
fn original_visual_material() {
    let sdf = load_str(
        include_str!("fixtures/material.sdf"),
        "material.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().unwrap();
    let link = model.links.first().unwrap();
    let vis1 = link.visuals.first().unwrap();
    let mat = vis1.material.as_deref().unwrap();
    color(mat.ambient, [0.4, 0.2, 0.3, 1.]);
    color(mat.diffuse, [0.2, 0.5, 0.1, 1.]);
    color(mat.specular, [0.7, 0.3, 0.5, 0.9]);
    color(mat.emissive, [1., 0., 0.2, 1.]);
    assert!(!mat.lighting);
    assert!(mat.double_sided);
    assert!(mat.render_order.to_bits().abs_diff(5.1_f32.to_bits()) <= 4);
    assert_eq!(mat.shader, ShaderType::Vertex);
    let script = mat.script.as_ref().unwrap();
    assert_eq!(script.uris[0], "myuri");
    assert_eq!(script.name, "myname");
    let vis2 = link.visuals.get(1).unwrap();
    let mat2 = vis2.material.as_deref().unwrap();
    assert_eq!(mat2.shader, ShaderType::Pixel);
    let vis3 = link.visuals.get(2).unwrap();
    let mat3 = vis3.material.as_deref().unwrap();
    assert_eq!(mat3.shader, ShaderType::NormalMapObjectSpace);
    assert_eq!(mat3.normal_map, "my_normal_map");
}
