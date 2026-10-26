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
// Observable member rename/relookup values from two DOMModel originals.
use sdformat::{Frame, Model, NoResources, Pose, Source, load_str};

fn model(name: &str, namespace: Option<&str>) -> Model {
    Model {
        name: name.into(),
        scoped_name: name.into(),
        raw_namespace: namespace.map(str::to_owned),
        pose: Pose::default(),
        is_static: false,
        canonical_link: String::new(),
        placement_frame: String::new(),
        include: None,
        merged_includes: Vec::new(),
        links: Vec::new(),
        joints: Vec::new(),
        models: Vec::new(),
        frames: Vec::new(),
        plugins: Vec::new(),
        source: Source {
            uri: "owned-model".into(),
            line: 1,
            column: 1,
        },
    }
}

#[test]
fn original_dom_model_copy_constructor() {
    let original = model("test_model", Some("test_ns"));
    let copied = original.clone();
    assert_eq!(copied.name, "test_model");
    let namespace = copied.namespace().expect("Namespace().has_value()");
    assert_eq!(namespace, "test_ns");
}

#[test]
fn original_dom_model_copy_assignment_operator() {
    let original = model("test_model", Some("test_ns"));
    let mut copied = model("", None);
    copied.clone_from(&original);
    assert_eq!(copied.name, "test_model");
    let namespace = copied.namespace().expect("Namespace().has_value()");
    assert_eq!(namespace, "test_ns");
}

#[test]
fn original_dom_model_move_constructor() {
    let original = model("test_model", Some("test_ns"));
    let moved = original;
    assert_eq!(moved.name, "test_model");
    let namespace = moved.namespace().expect("Namespace().has_value()");
    assert_eq!(namespace, "test_ns");
}

#[test]
fn original_dom_model_move_assignment_operator() {
    let original = model("test_model", Some("test_ns"));
    let mut moved = model("", None);
    let previous = std::mem::replace(&mut moved, original);
    assert_eq!(moved.name, "test_model");
    let namespace = moved.namespace().expect("Namespace().has_value()");
    assert_eq!(namespace, "test_ns");
    assert!(previous.name.is_empty()); // Supplemental replacement control.
    assert!(previous.raw_namespace.is_none());
}

#[test]
fn original_dom_model_copy_assignment_after_move() {
    let first = model("model1", Some("test_ns1"));
    let second = model("model2", Some("test_ns2"));
    let temporary = first;
    let first = second.clone();
    let second = temporary.clone();
    assert_eq!(first.name, "model2");
    assert_eq!(second.name, "model1");
    let first_namespace = first.namespace().expect("Namespace().has_value()");
    assert_eq!(first_namespace, "test_ns2");
    let second_namespace = second.namespace().expect("Namespace().has_value()");
    assert_eq!(second_namespace, "test_ns1");
}

#[test]
fn namespace_metadata_preserves_absence_empty_raw_and_name_changes() {
    // Namespace observations from Construction are supplemental, not a whole-case port.
    let mut value = model("", None);
    assert!(value.raw_namespace.is_none());
    assert!(value.namespace().is_none());
    value.name = "test_name".into();
    value.raw_namespace = Some(String::new());
    assert_eq!(value.namespace().as_deref(), Some(""));
    value.raw_namespace = Some("test/{name}1/{name}2/test_ns".into());
    assert_eq!(
        value.namespace().as_deref(),
        Some("test/test_name1/test_name2/test_ns")
    );
    value.name = "test_model".into();
    assert_eq!(
        value.namespace().as_deref(),
        Some("test/test_model1/test_model2/test_ns")
    );
    assert_eq!(
        value.raw_namespace.as_deref(),
        Some("test/{name}1/{name}2/test_ns")
    );
    value.raw_namespace = Some("test_namespace".into());
    assert_eq!(value.namespace().as_deref(), Some("test_namespace"));

    // Inserted model names are not expanded again, even when they contain the token.
    value.name = "機体{name}".into();
    value.raw_namespace = Some("/{name}{name}/{Name}".into());
    assert_eq!(
        value.namespace().as_deref(),
        Some("/機体{name}機体{name}/{Name}")
    );
    value.name.clear();
    assert_eq!(value.namespace().as_deref(), Some("//{Name}"));
    value.raw_namespace = None;
    assert!(value.namespace().is_none());
}

#[test]
fn nested_model_and_frame_clones_survive_independent_edits_and_drop() {
    let xml = "<sdf version='1.12'><model name='parent'><link name='base'/><frame name='mount' attached_to='base'/><model name='child'><link name='part'/></model><plugin name='control' filename='controller'><setting>original</setting></plugin></model></sdf>";
    let mut original = load_str(xml, "nested-owned.sdf", &NoResources)
        .unwrap()
        .models
        .remove(0);
    assert!(original.raw_namespace.is_none());
    original.raw_namespace = Some("ns/{name}".into());
    original.models[0].raw_namespace = Some("child/{name}".into());
    original.frames[0].pose.relative_to = "base".into();
    let copied = original.clone();
    original.name = "changed".into();
    original.raw_namespace.as_mut().unwrap().clear();
    original.links[0].name.clear();
    original.models[0].name.clear();
    original.models[0].raw_namespace.as_mut().unwrap().clear();
    original.frames[0].name.clear();
    original.frames[0].pose.relative_to.clear();
    original.frames[0].source.uri.clear();
    original.plugins[0].xml.clear();
    drop(original);
    assert_eq!(copied.namespace().as_deref(), Some("ns/parent"));
    assert_eq!(copied.links[0].name, "base");
    assert_eq!(copied.models[0].namespace().as_deref(), Some("child/child"));
    assert_eq!(copied.models[0].links[0].name, "part");
    assert_eq!(copied.frames[0].name, "mount");
    assert_eq!(copied.frames[0].attached_to, "base");
    assert_eq!(copied.frames[0].pose.relative_to, "base");
    assert_eq!(copied.frames[0].source.uri, "nested-owned.sdf");
    assert!(
        copied.plugins[0]
            .xml
            .contains("<setting>original</setting>")
    );
    let frame: Frame = copied.frames[0].clone();
    drop(copied);
    assert_eq!(frame.name, "mount");
    assert_eq!(frame.pose.relative_to, "base");
}

#[test]
fn xml_empty_namespace_is_absent_while_owned_empty_is_present() {
    let xml =
        "<sdf version='1.12'><model name='robot' namespace=''><link name='base'/></model></sdf>";
    let mut parsed = load_str(xml, "namespace-empty.sdf", &NoResources)
        .unwrap()
        .models
        .remove(0);
    assert!(parsed.raw_namespace.is_none());
    assert!(parsed.namespace().is_none());
    parsed.raw_namespace = Some(String::new());
    assert_eq!(parsed.namespace().as_deref(), Some(""));
}
