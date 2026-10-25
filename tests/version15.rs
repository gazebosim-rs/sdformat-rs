// Copyright (C) 2026 Open Source Robotics Foundation
// Copyright 2015 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Ports: sdformat/test/integration/nested_model.cc,
// NestedModel.IncludeElements.
// IncludeElements uses the exact original raw-string input and unchanged resources.
use sdformat::*;
struct OriginalResources;
impl ResourceResolver for OriginalResources {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        assert_eq!(parent, "include-elements");
        let xml = match uri {
            "box" => include_str!("fixtures/include_elements_box.sdf"),
            "test_model" => include_str!("fixtures/include_elements_model.sdf"),
            _ => panic!("unexpected URI {uri}"),
        };
        Ok(Resource {
            canonical_uri: format!("fixture:{uri}/model.sdf"),
            xml: xml.into(),
        })
    }
}
fn original_include(model: &Model, uri: &str, name: &str) {
    let include = model.include.as_ref().unwrap();
    assert_eq!(include.uri, uri);
    assert_eq!(include.name.as_deref(), Some(name));
    let pose = &include.pose.as_ref().unwrap().transform;
    for (actual, expected) in pose
        .translation
        .into_iter()
        .chain(pose.rotation)
        .zip([1., 2., 3., 0., 0., 0., 1.])
    {
        assert!((actual - expected).abs() <= 1e-3);
    }
    assert_eq!(include.placement_frame.as_deref(), Some("link"));
    let plugin = include.plugins.first().unwrap();
    assert_eq!(plugin.name, "test_plugin");
    assert_eq!(plugin.filename, "test_plugin_file");
    assert!(plugin.xml.contains("name='test_plugin'"));
    assert!(plugin.xml.contains("filename='test_plugin_file'"));
}
#[test]
fn nested_model_include_elements_original_assertions() {
    let sdf = load_str(
        include_str!("fixtures/include_elements.sdf"),
        "include-elements",
        &OriginalResources,
    )
    .unwrap();
    let world = sdf.worlds.first().unwrap();
    let first = world.models.first().unwrap();
    original_include(first, "box", "test_box");
    let second = world.models.get(1).unwrap();
    assert!(second.include.is_none());
    let nested = second.models.first().unwrap();
    original_include(nested, "test_model", "test_model");
}
#[test]
fn unimplemented_15_conversion_actions_fail_explicitly() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("version15_unsupported.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let xml = case["xml"].as_str().unwrap();
        assert_eq!(
            load_str(xml, "unsupported-conversion", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}
