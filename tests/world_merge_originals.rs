/*
 * Copyright 2019 Open Source Robotics Foundation
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
// Faithful predicates from IncludesTest.WorldMergeInclude and InvalidElements.
use sdformat::{ErrorCode, Resource, ResourceResolver, Transform, load_str};

struct Fixtures;
impl ResourceResolver for Fixtures {
    fn resolve(&self, uri: &str, _parent: &str) -> Result<Resource, String> {
        let (canonical, xml) = match uri {
            "model_for_world_merge_include.sdf" => (
                "resolved://merge/model.sdf",
                include_str!("fixtures/model_for_world_merge_include.sdf"),
            ),
            "test_model" => (
                "resolved://test_model/model.sdf",
                include_str!("fixtures/root_include_test_model.sdf"),
            ),
            _ => return Err(format!("unknown fixture {uri}")),
        };
        Ok(Resource {
            canonical_uri: canonical.into(),
            xml: xml.into(),
        })
    }
}
#[test]
fn original_includes_test_world_merge_include() {
    let root = load_str(
        include_str!("fixtures/world_merge_include.sdf"),
        "world_merge_include.sdf",
        &Fixtures,
    )
    .expect("errors.empty()");
    let world = root.worlds.first().expect("WorldByIndex(0) != nullptr");
    let proxy_name = "_merged__model_for_world_merge_include__model__";
    let proxy = world
        .frames
        .iter()
        .find(|f| f.name == proxy_name)
        .expect("proxyFrame != nullptr");
    assert_eq!(
        proxy.pose.transform,
        Transform {
            translation: [100., 0., 0.],
            rotation: [0., 0., 0., 1.]
        }
    );
    assert_eq!(world.models.len(), 2);
    let model1 = world.models.first().expect("ModelByIndex(0) != nullptr");
    assert_eq!(model1.pose.relative_to, proxy_name);
    let model2 = world.models.get(1).expect("ModelByIndex(1) != nullptr");
    assert_eq!(model2.pose.relative_to, proxy_name);
    let frame1 = world
        .frames
        .iter()
        .find(|f| f.name == "F1")
        .expect("FrameByName(F1) != nullptr");
    assert_eq!(frame1.attached_to, proxy_name);
    let _joint1 = world
        .joints
        .iter()
        .find(|j| j.name == "J1")
        .expect("JointByName(J1) != nullptr");
}
#[test]
fn original_includes_test_world_merge_include_invalid_elements() {
    let xml = r#"
  <sdf version="1.10">
    <world name="invalid_world">
      <include merge="true">
        <uri>test_model</uri>
      </include>
    </world>
  </sdf>"#;
    let error = load_str(xml, "invalid_world.sdf", &Fixtures).expect_err("!errors.empty()");
    assert_eq!(error.code, ErrorCode::MergeIncludeUnsupported);
}
