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
use sdformat::{Frame, Model, NoResources, Sdf, load_str};

struct Tokens<'a>(std::slice::Iter<'a, serde_json::Value>);
impl Tokens<'_> {
    fn next(&mut self) -> &str {
        self.0.next().unwrap().as_str().unwrap()
    }
    fn count(&mut self) -> usize {
        self.next().parse().unwrap()
    }
    fn text(&mut self) -> String {
        let value = self.next();
        if value == "-" {
            return String::new();
        }
        String::from_utf8(
            (0..value.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
                .collect(),
        )
        .unwrap()
    }
    fn ok(&mut self) {
        assert_eq!(self.count(), 0);
    }
}
fn native_body(sdf: &Sdf, name: &str, scope: &str, tokens: &mut Tokens<'_>) {
    tokens.ok();
    let native = tokens.text();
    let expected = if native == "world" {
        if sdf.worlds.is_empty() {
            "::".into()
        } else {
            sdf.worlds[0].name.clone()
        }
    } else if scope.is_empty() {
        native
    } else {
        format!("{scope}::{native}")
    };
    assert_eq!(sdf.attached_body(name).unwrap(), expected, "{name}");
}
fn frames(sdf: &Sdf, frames: &[Frame], scope: &str, tokens: &mut Tokens<'_>) {
    assert_eq!(tokens.count(), frames.len());
    for frame in frames {
        assert_eq!(tokens.text(), frame.name);
        let name = format!("{scope}::{}", frame.name);
        native_body(sdf, &name, scope, tokens);
        tokens.ok();
        let pose = sdf.pose_of(&name, scope).unwrap();
        for actual in pose.translation.into_iter().chain(pose.rotation) {
            let expected = f64::from_bits(tokens.next().parse().unwrap());
            assert!(
                (actual - expected).abs() <= 1e-12,
                "{name}: {actual} vs {expected}"
            );
        }
    }
}
fn model(sdf: &Sdf, value: &Model, world: &str, tokens: &mut Tokens<'_>) {
    assert_eq!(tokens.text(), value.name);
    assert_eq!(tokens.count() != 0, value.is_static);
    native_body(sdf, &value.scoped_name, world, tokens);
    let (link, name) = value.canonical_link_and_relative_name();
    assert_eq!(tokens.count() != 0, link.is_some());
    assert_eq!(tokens.text(), name);
    frames(sdf, &value.frames, &value.scoped_name, tokens);
    assert_eq!(tokens.count(), value.joints.len());
    for joint in &value.joints {
        assert_eq!(tokens.text(), joint.name);
        assert_eq!(tokens.text(), joint.parent);
        assert_eq!(tokens.text(), joint.child);
        for endpoint in [&joint.parent, &joint.child] {
            let name = sdf.resolve_name(&value.scoped_name, endpoint).unwrap();
            native_body(sdf, &name, &value.scoped_name, tokens);
        }
    }
    assert_eq!(tokens.count(), value.models.len());
    for nested in &value.models {
        model(sdf, nested, world, tokens);
    }
}
#[test]
fn native_static_attachment_and_joint_endpoint_corpus() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("static_model.json")).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 61);
    for row in rows.as_array().unwrap() {
        let parsed = load_str(
            row["xml"].as_str().unwrap(),
            row["id"].as_str().unwrap(),
            &NoResources,
        );
        let mut tokens = Tokens(row["tokens"].as_array().unwrap().iter());
        let errors = tokens.count();
        assert_eq!(
            parsed.is_ok(),
            errors == 0,
            "{}: {:?}",
            row["id"],
            parsed.as_ref().err()
        );
        if errors != 0 {
            continue;
        }
        let sdf = parsed.unwrap();
        assert_eq!(tokens.count(), sdf.models.len());
        for value in &sdf.models {
            tokens.ok();
            tokens.ok(); // Independent native graph build and validation.
            model(&sdf, value, "", &mut tokens);
        }
        assert_eq!(tokens.count(), sdf.worlds.len());
        for world in &sdf.worlds {
            assert_eq!(tokens.text(), world.name);
            frames(&sdf, &world.frames, &world.name, &mut tokens);
            tokens.ok();
            tokens.ok();
            assert_eq!(tokens.count(), world.models.len());
            for value in &world.models {
                model(&sdf, value, &world.name, &mut tokens);
            }
        }
        assert!(tokens.0.next().is_none(), "{}", row["id"]);
    }
}
#[test]
fn original_model_without_links_with_nested_static_model() {
    let loaded = load_str(
        include_str!("fixtures/model_nested_static_model.sdf"),
        "model_nested_static_model.sdf",
        &NoResources,
    );
    // The typed loader eagerly builds and validates both graphs before success.
    assert!(loaded.is_ok(), "{:?}", loaded.as_ref().err());
    let sdf = loaded.unwrap();
    let model = sdf.models.first();
    assert!(model.is_some());
    let resolved = sdf.attached_body(&model.unwrap().scoped_name);
    assert!(resolved.is_ok());
    assert_eq!(
        resolved.unwrap(),
        "model_nested_static_model::child_model::static_model::__model__"
    );
}
