/*
 * Copyright 2017 Open Source Robotics Foundation
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

/*
 * Copyright (C) 2019 Open Source Robotics Foundation
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
use sdformat::{ErrorCode, NoResources, Scene, load_str};
fn scene(xml: &str) -> Result<Scene, sdformat::Diagnostic> {
    let sdf = load_str(xml, "scene.sdf", &NoResources)?;
    Ok(sdf.worlds[0].scene.clone())
}
fn world(payload: &str) -> String {
    format!("<sdf version='1.12'><world name='w'>{payload}</world></sdf>")
}
#[test]
fn world_scene_defaults_and_explicit_values_match_native() {
    let default = Scene::default();
    assert_eq!(
        default.ambient.map(f32::to_bits),
        [1053609165, 1053609165, 1053609165, 1065353216]
    );
    assert_eq!(
        default.background.map(f32::to_bits),
        [1060320051, 1060320051, 1060320051, 1065353216]
    );
    assert!(default.grid && default.shadows && default.origin_visual);
    assert_eq!(default.source, None);
    let missing = scene(&world("")).unwrap();
    assert_eq!(missing, default);
    let explicit = scene(&world("<scene/>")).unwrap();
    assert_eq!(explicit.ambient, default.ambient);
    assert_eq!(explicit.background, default.background);
    assert!(explicit.grid && explicit.shadows && explicit.origin_visual);
    assert_eq!(explicit.source.as_ref().unwrap().uri, "scene.sdf");
    assert_eq!(explicit.source.as_ref().unwrap().line, 1);
    assert_eq!(explicit.source.as_ref().unwrap().column, 37);
    let configured = scene(&world("<scene><ambient>0.1 0.2 0.3 0.4</ambient><background>0.8 0.7 0.6 0.5</background><grid>false</grid><shadows>0</shadows><origin_visual>FALSE</origin_visual></scene>")).unwrap();
    assert_eq!(configured.ambient, [0.1, 0.2, 0.3, 0.4]);
    assert_eq!(configured.background, [0.8, 0.7, 0.6, 0.5]);
    assert!(!configured.grid && !configured.shadows && !configured.origin_visual);
}
#[test]
fn exact_original_scene_subtree_retains_all_five_missing_world_observations() {
    let xml = include_str!("fixtures/world_scene_original.sdf");
    let sdf = load_str(xml, "world_complete.sdf#scene", &NoResources).unwrap();
    assert_eq!(sdf.worlds.len(), 1);
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "default");
    assert_eq!(world.scene.ambient, [0.3, 0.4, 0.5, 1.]);
    assert_eq!(world.scene.background, [0.6, 0.7, 0.8, 1.]);
    assert!(world.scene.grid);
    assert!(world.scene.shadows);
    assert!(world.scene.origin_visual);
    let source = world.scene.source.as_ref().unwrap();
    assert_eq!(source.uri, "world_complete.sdf#scene");
    assert_eq!((source.line, source.column), (2, 5));
    let mut snapshot = world.scene.clone();
    snapshot.ambient[0] = 0.9;
    snapshot.source.as_mut().unwrap().uri.clear();
    assert_eq!(world.scene.ambient, [0.3, 0.4, 0.5, 1.]);
    assert_eq!(
        world.scene.source.as_ref().unwrap().uri,
        "world_complete.sdf#scene"
    );
    let debug = format!("{world:?}");
    assert!(debug.contains("ambient: [0.3, 0.4, 0.5, 1.0]"));
    assert!(debug.contains("world_complete.sdf#scene"));
}
#[test]
fn required_empty_values_and_unsupported_scene_domains_remain_explicit_errors() {
    for tag in ["ambient", "background", "shadows"] {
        for text in ["", " \t\n "] {
            let xml = world(&format!("<scene><{tag}>{text}</{tag}></scene>"));
            assert_eq!(
                scene(&xml).unwrap_err().code,
                ErrorCode::InvalidValue,
                "{tag}"
            );
        }
    }
    for tag in ["grid", "origin_visual"] {
        let value = scene(&world(&format!("<scene><{tag}/></scene>"))).unwrap();
        assert!(value.grid && value.origin_visual);
    }
    for payload in [
        "<scene><sky/></scene>",
        "<scene><fog/></scene>",
        "<scene extra='x'/>",
        "<scene><other/></scene>",
        "<gui/>",
        "<Scene/>",
    ] {
        assert_eq!(
            scene(&world(payload)).unwrap_err().code,
            ErrorCode::Unsupported,
            "{payload}"
        );
    }
    for payload in [
        "<scene/><scene/>",
        "<scene><grid>true</grid><grid>false</grid></scene>",
    ] {
        assert_eq!(
            scene(&world(payload)).unwrap_err().code,
            ErrorCode::InvalidValue
        );
    }
}

fn compare_native(scene: &Scene, output: &str) {
    let line = output
        .lines()
        .find(|line| line.starts_with("VALUES "))
        .unwrap();
    let expected: Vec<u32> = line
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect();
    let mut actual: Vec<u32> = scene
        .ambient
        .into_iter()
        .chain(scene.background)
        .map(f32::to_bits)
        .collect();
    actual.extend([
        u32::from(scene.grid),
        u32::from(scene.shadows),
        u32::from(scene.origin_visual),
    ]);
    assert_eq!(actual, expected[..11]);
}
#[test]
fn native_scene_matrix_preserves_values_and_explicit_policy_boundaries() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!("world_scene.json")).unwrap();
    for row in corpus["cases"].as_array().unwrap() {
        let loaded = scene(row["xml"].as_str().unwrap());
        match row["rust_policy"].as_str().unwrap() {
            "accepted" => compare_native(
                &loaded.unwrap_or_else(|error| panic!("{}: {error:?}", row["name"])),
                row["stdout"].as_str().unwrap(),
            ),
            "Unsupported" => assert_eq!(
                loaded.unwrap_err().code,
                ErrorCode::Unsupported,
                "{}",
                row["name"]
            ),
            "InvalidValue" => assert_eq!(
                loaded.unwrap_err().code,
                ErrorCode::InvalidValue,
                "{}",
                row["name"]
            ),
            _ => unreachable!(),
        }
    }
}
