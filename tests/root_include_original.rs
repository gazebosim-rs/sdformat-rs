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
// Six assertions from IncludesTest.IncludeModelEmptyNamespace.
use sdformat::{Resource, ResourceResolver, load_str};

struct ExactModel;
impl ResourceResolver for ExactModel {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        assert_eq!(uri, "test_model");
        assert_eq!(parent, "root-include.sdf");
        Ok(Resource {
            canonical_uri: "resolved://test_model/model.sdf".into(),
            xml: include_str!("fixtures/root_include_test_model.sdf").into(),
        })
    }
}

#[test]
fn original_includes_test_include_model_empty_namespace() {
    let xml =
        "<sdf version='1.12'><include><uri>test_model</uri><namespace></namespace></include></sdf>";
    let root = load_str(xml, "root-include.sdf", &ExactModel).expect("errors.empty()");
    let model = root.models.first().expect("Root.Model() != nullptr");
    assert_eq!(model.name, "test_model");
    assert!(model.namespace().is_none());
    assert!(model.raw_namespace.is_none());
    assert_eq!(model.links.len(), 1);
}
