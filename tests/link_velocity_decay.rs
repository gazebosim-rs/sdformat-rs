/*
 * Copyright 2015 Open Source Robotics Foundation
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
use sdformat::{ErrorCode, NoResources, load_str};
use serde_json::Value;
fn data() -> Value {
    serde_json::from_str(include_str!("link_velocity_decay.json")).unwrap()
}
fn hex(value: &str) -> String {
    format!(
        "{}:{}",
        value.len(),
        value
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}
fn metadata_stream(xml: &str) -> Result<String, sdformat::Diagnostic> {
    let sdf = load_str(xml, "decay.sdf", &NoResources)?;
    let model = &sdf.models[0];
    let mut result = format!(
        "ERRORS 0\nMODEL {} LINKS {}\n",
        hex(&model.name),
        model.links.len()
    );
    for link in &model.links {
        result += &format!(
            "LINK {} DECAY {}\n",
            hex(&link.name),
            u8::from(link.velocity_decay.is_some())
        );
        if let Some(value) = &link.velocity_decay {
            result += &format!(
                "VALUES {} {}\n",
                value.linear.to_bits(),
                value.angular.to_bits()
            );
            assert_eq!(value.source.uri, "decay.sdf");
            assert!(xml.contains(&value.xml));
        }
    }
    Ok(result)
}
fn effective_values(stdout: &str) -> String {
    stdout
        .lines()
        .map(|line| {
            if let Some((_, values)) = line.split_once(" VALUES ") {
                format!("VALUES {values}")
            } else {
                line.into()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
#[test]
fn xml_decay_metadata_matches_native_defaults_domain_and_errors() {
    for row in data()["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let result = metadata_stream(row["xml"].as_str().unwrap());
        match row["rust_policy"].as_str().unwrap() {
            "accepted" => assert_eq!(
                result.unwrap_or_else(|e| panic!("{name}: {e:?}")),
                effective_values(row["stdout"].as_str().unwrap()),
                "{name}"
            ),
            policy => assert_eq!(
                result.unwrap_err().code,
                match policy {
                    "InvalidValue" => ErrorCode::InvalidValue,
                    "Unsupported" => ErrorCode::Unsupported,
                    _ => panic!("unknown policy {policy}"),
                },
                "{name}"
            ),
        }
    }
}
#[test]
fn original_urdf_extension_velocity_decay_xml_subset() {
    let xml = include_str!("fixtures/link_velocity_decay_native.sdf");
    let sdf = load_str(xml, "native-urdf-projection.sdf", &NoResources).unwrap();
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!(model.name, "urdf_gazebo_extension_test");
    assert_eq!(model.links.len(), 6);
    for (link, name) in model
        .links
        .iter()
        .zip(["link0", "link1", "link2", "link3", "link4", "link1a"])
    {
        assert_eq!(link.name, name);
        if matches!(name, "link1" | "link2") {
            let value = link.velocity_decay.as_ref().unwrap();
            assert!(value.linear.to_bits().abs_diff(0.1_f64.to_bits()) <= 4);
            let angular = if name == "link1" { 0.2_f64 } else { 0.1 };
            assert!(value.angular.to_bits().abs_diff(angular.to_bits()) <= 4);
            assert_eq!(value.source.uri, "native-urdf-projection.sdf");
            assert_eq!(
                (value.source.line, value.source.column),
                (if name == "link1" { 6 } else { 12 }, 7)
            );
            assert!(value.xml.contains("<linear>0.10000000000000001</linear>"));
        } else {
            assert_eq!(link.velocity_decay, None);
        }
    }
    assert_eq!(
        metadata_stream(xml).unwrap(),
        effective_values(data()["original_urdf"]["stdout"].as_str().unwrap())
    );
}
#[test]
fn owned_link_decay_snapshot_clones_without_element_aliasing_claims() {
    let original = {
        let xml = String::from(
            "<sdf version='1.12'><model name='m'><link name='l'><velocity_decay><linear>-1</linear><angular>25.2</angular></velocity_decay></link></model></sdf>",
        );
        load_str(&xml, "owned-decay.sdf", &NoResources)
            .unwrap()
            .models
            .remove(0)
            .links
            .remove(0)
    };
    let mut copy = original.clone();
    assert_eq!(copy.velocity_decay, original.velocity_decay);
    let copied = copy.velocity_decay.as_mut().unwrap();
    copied.linear = 0.75;
    copied.source.uri.clear();
    copied.xml.clear();
    let value = original.velocity_decay.as_ref().unwrap();
    assert_eq!(value.linear, -1.);
    assert_eq!(value.angular, 25.2);
    assert_eq!(value.source.uri, "owned-decay.sdf");
    assert_eq!((value.source.line, value.source.column), (1, 52));
    assert_eq!(
        value.xml,
        "<velocity_decay><linear>-1</linear><angular>25.2</angular></velocity_decay>"
    );
    drop(original);
    assert_eq!(copy.velocity_decay.as_ref().unwrap().linear, 0.75);
    assert!(format!("{copy:?}").contains("VelocityDecayMetadata"));
}
#[test]
fn decay_payload_and_scope_guards_remain_explicit() {
    for (payload, code) in [
        ("<velocity_decay extra='x'/>", ErrorCode::Unsupported),
        (
            "<velocity_decay><extra/></velocity_decay>",
            ErrorCode::Unsupported,
        ),
        (
            "<velocity_decay><linear extra='x'/></velocity_decay>",
            ErrorCode::Unsupported,
        ),
        (
            "<velocity_decay><angular><extra/></angular></velocity_decay>",
            ErrorCode::Unsupported,
        ),
        (
            "<velocity_decay><angular>bad</angular></velocity_decay>",
            ErrorCode::InvalidValue,
        ),
        (
            "<velocity_decay><angular>0</angular><angular>1</angular></velocity_decay>",
            ErrorCode::InvalidValue,
        ),
        ("<Velocity_Decay/>", ErrorCode::Unsupported),
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'>{payload}</link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "decay.sdf", &NoResources).unwrap_err().code,
            code,
            "{payload}"
        );
    }
    let world = "<sdf version='1.12'><world name='w'><velocity_decay/></world></sdf>";
    assert_eq!(
        load_str(world, "wrong-scope.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
    let model =
        "<sdf version='1.12'><model name='m'><velocity_decay/><link name='l'/></model></sdf>";
    let sdf = load_str(model, "opaque-model.sdf", &NoResources).unwrap();
    assert_eq!(sdf.models[0].links[0].velocity_decay, None);
    assert_eq!(sdf.models[0].extensions.len(), 1);
    assert_eq!(sdf.models[0].extensions[0].tag, "velocity_decay");
    assert_eq!(sdf.models[0].extensions[0].xml, "<velocity_decay/>");
    assert_eq!(sdf.models[0].extensions[0].source.uri, "opaque-model.sdf");
}
#[test]
fn malformed_retained_linear_scalar_reports_actual_input_position() {
    let xml = include_str!("fixtures/bad_syntax_double.sdf");
    let error = load_str(xml, "bad_syntax_double.sdf", &NoResources).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!(
        error.source,
        sdformat::Source {
            uri: "bad_syntax_double.sdf".into(),
            line: 7,
            column: 11
        }
    );
}
