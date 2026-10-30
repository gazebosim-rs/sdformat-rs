/*
 * Copyright (C) 2020 Open Source Robotics Foundation
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
fn double_eq(a: f64, b: f64) {
    assert!(a.to_bits().abs_diff(b.to_bits()) <= 4, "{a} != {b}");
}
fn vector(a: [f64; 3], b: [f64; 3]) {
    for (a, b) in a.into_iter().zip(b) {
        assert!((a - b).abs() <= 1e-3);
    }
}
fn check() {
    let mut heightmap = load_heightmap_str("<heightmap/>", "configured.xml").unwrap();
    heightmap.uri = "https://test-uri.org".into();
    heightmap.size = [1., 2., 3.];
    heightmap.position = [4., 5., 6.];
    heightmap.use_terrain_paging = true;
    heightmap.sampling = 2;
    heightmap.textures.push(HeightmapTexture {
        size: 1.2,
        diffuse: "diffuse_map".into(),
        normal: "normal_map".into(),
    });
    heightmap.blends.push(HeightmapBlend {
        min_height: 1.2,
        fade_distance: 3.4,
    });
    let xml = heightmap.to_xml().unwrap();
    assert!(!xml.is_empty());
    let loaded = load_heightmap_str(&xml, "heightmap-original.xml").unwrap();
    assert_eq!(heightmap.uri, loaded.uri);
    vector(heightmap.size, loaded.size);
    vector(heightmap.position, loaded.position);
    assert_eq!(heightmap.use_terrain_paging, loaded.use_terrain_paging);
    assert_eq!(heightmap.sampling, loaded.sampling);
    assert_eq!(heightmap.textures.len(), loaded.textures.len());
    assert_eq!(loaded.textures.len(), 1);
    double_eq(heightmap.textures[0].size, loaded.textures[0].size);
    assert_eq!(heightmap.textures[0].diffuse, loaded.textures[0].diffuse);
    assert_eq!(heightmap.textures[0].normal, loaded.textures[0].normal);
    assert_eq!(heightmap.blends.len(), loaded.blends.len());
    assert_eq!(loaded.blends.len(), 1);
    double_eq(heightmap.blends[0].min_height, loaded.blends[0].min_height);
    double_eq(
        heightmap.blends[0].fade_distance,
        loaded.blends[0].fade_distance,
    );
}
#[test]
fn original_heightmap_to_element() {
    check();
}
#[test]
fn original_heightmap_to_element_error_output() {
    quiet::assert_quiet(
        "original_heightmap_to_element_error_output",
        "SDF_HEIGHTMAP_QUIET_CHILD",
        check,
    );
}
