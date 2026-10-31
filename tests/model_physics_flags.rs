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
use sdformat::{ErrorCode, Model, NoResources, Resource, ResourceResolver, load_str};
use serde_json::Value;
#[path = "support/model_physics_flags_projection.rs"]
mod projection;

fn data() -> Value {
    serde_json::from_str(include_str!("model_physics_flags.json")).unwrap()
}
struct Fixture<'a>(Option<&'a str>);
impl ResourceResolver for Fixture<'_> {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        assert_eq!(uri, "fixture.sdf");
        assert_eq!(parent, "flags.sdf");
        Ok(Resource {
            canonical_uri: "resolved://flags/child.sdf".into(),
            xml: self.0.unwrap().into(),
        })
    }
}
fn model_stream(model: &Model, result: &mut String) {
    *result += &format!(
        "MODEL {} {} {} {} {}\n",
        model.scoped_name,
        u8::from(model.self_collide),
        u8::from(model.allow_auto_disable),
        u8::from(model.is_static),
        u8::from(model.enable_wind)
    );
    for child in &model.models {
        model_stream(child, result);
    }
}
#[test]
fn native_model_flag_defaults_versions_boolean_domain_and_scope() {
    for row in data()["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        let result = load_str(xml, "flags.sdf", &Fixture(row["resource"].as_str()));
        match row["rust_policy"].as_str().unwrap() {
            "accepted" | "opaque" => {
                let sdf = result.unwrap_or_else(|e| panic!("{name}: {e:?}"));
                let mut actual = String::from("ERRORS 0\n");
                for model in &sdf.models {
                    model_stream(model, &mut actual);
                }
                for world in &sdf.worlds {
                    for model in &world.models {
                        model_stream(model, &mut actual);
                    }
                }
                assert_eq!(actual, row["stdout"].as_str().unwrap(), "{name}");
                if row["rust_policy"] == "opaque" {
                    assert_eq!(sdf.models[0].extensions.len(), 1);
                    assert_eq!(sdf.models[0].extensions[0].source.uri, "flags.sdf");
                }
            }
            policy => {
                let error = result.expect_err(name);
                assert_eq!(
                    error.code,
                    if policy == "InvalidValue" {
                        ErrorCode::InvalidValue
                    } else {
                        assert_eq!(policy, "Unsupported");
                        ErrorCode::Unsupported
                    },
                    "{name}"
                );
                assert_eq!(error.source.uri, "flags.sdf");
            }
        }
    }
}
#[test]
fn original_model_construction_flag_value_subset() {
    let mut model = load_str(
        "<sdf version='1.12'><model name='m'><link name='l'/></model></sdf>",
        "defaults.sdf",
        &NoResources,
    )
    .unwrap()
    .models
    .remove(0);
    assert!(!model.self_collide);
    model.self_collide = true;
    assert!(model.self_collide);
    assert!(model.allow_auto_disable);
    model.allow_auto_disable = false;
    assert!(!model.allow_auto_disable);
    let copy = model.clone();
    model.self_collide = false;
    model.allow_auto_disable = true;
    assert!(copy.self_collide && !copy.allow_auto_disable);
    assert!(!model.self_collide && model.allow_auto_disable);
    assert_eq!(
        data()["constructor"]["stdout"],
        "MODEL default 0 1 0 0\nMODEL set 1 0 0 0\nMODEL original 0 1 0 0\nMODEL copy 1 0 0 0\n"
    );
}
#[test]
fn owned_flags_preserve_source_loaded_snapshot_and_child_independence() {
    let original = {
        let xml = String::from(
            "<sdf version='1.12'>\n<model name='parent'><self_collide>true</self_collide><allow_auto_disable>false</allow_auto_disable><link name='l'/><model name='child'><link name='c'/></model></model></sdf>",
        );
        load_str(&xml, "owned-flags.sdf", &NoResources)
            .unwrap()
            .models
            .remove(0)
    };
    assert!(original.self_collide && !original.allow_auto_disable);
    assert!(!original.models[0].self_collide && original.models[0].allow_auto_disable);
    assert!(!original.links[0].properties.self_collide);
    assert_eq!((original.source.line, original.source.column), (2, 1));
    let snapshot = original.loaded_element.clone().unwrap();
    assert!(snapshot.has_child("self_collide") && snapshot.has_child("allow_auto_disable"));
    let mut copy = original.clone();
    copy.self_collide = false;
    copy.allow_auto_disable = true;
    copy.source.uri.clear();
    assert_eq!(copy.loaded_element.as_ref(), Some(&snapshot));
    assert!(original.self_collide && !original.allow_auto_disable);
    assert_eq!(original.source.uri, "owned-flags.sdf");
    drop(original);
    assert!(!copy.self_collide && copy.allow_auto_disable);
    assert!(format!("{copy:?}").contains("allow_auto_disable"));
}
#[test]
fn nested_included_and_merged_flags_keep_each_models_source() {
    let input = data();
    for row in input["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r.get("resource").is_some())
    {
        let sdf = load_str(
            row["xml"].as_str().unwrap(),
            "flags.sdf",
            &Fixture(row["resource"].as_str()),
        )
        .unwrap();
        let parent = &sdf.models[0];
        assert_eq!(parent.source.uri, "flags.sdf");
        if row["name"].as_str().unwrap().contains("mergeFalse") {
            let child = &parent.models[0];
            assert_eq!(child.source.uri, "resolved://flags/child.sdf");
            assert_ne!(child.self_collide, parent.self_collide);
            assert_ne!(child.allow_auto_disable, parent.allow_auto_disable);
            assert!(
                child
                    .loaded_element
                    .as_ref()
                    .unwrap()
                    .has_child("self_collide")
            );
        } else {
            assert!(parent.models.is_empty());
            assert_eq!(parent.merged_includes.len(), 1);
            assert_eq!(parent.links.len(), 2);
            let declared = row["name"].as_str().unwrap().contains("parentTrue");
            for tag in ["self_collide", "allow_auto_disable"] {
                assert_eq!(
                    parent.loaded_element.as_ref().unwrap().has_child(tag),
                    declared
                );
            }
        }
    }
}
#[test]
fn prepared_same_input_uri_empty_schema_flags_are_complete_model_positives() {
    for tag in ["self_collide", "allow_auto_disable"] {
        let xml =
            format!("<sdf version='1.12'><model name='m'><link name='l'/><{tag}/></model></sdf>");
        let sdf = load_str(&xml, "root.sdf", &NoResources).unwrap();
        projection::assert_empty_flag_model(&sdf, &xml, "root.sdf", tag);
    }
}
#[test]
fn wrong_scope_and_invalid_boolean_source_remain_explicit() {
    let xml = "<sdf version='1.12'><model name='m'><link name='l'/>\n<allow_auto_disable>2</allow_auto_disable></model></sdf>";
    let error = load_str(xml, "invalid-flags.sdf", &NoResources).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!((error.source.line, error.source.column), (2, 1));
    for field in ["self_collide", "allow_auto_disable"] {
        let xml = format!("<sdf version='1.12'><world name='w'><{field}/></world></sdf>");
        assert_eq!(
            load_str(&xml, "world-flags.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
}
