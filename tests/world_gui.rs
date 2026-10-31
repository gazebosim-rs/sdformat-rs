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

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, load_str};
use serde_json::Value;

fn data() -> Value {
    serde_json::from_str(include_str!("world_gui.json")).unwrap()
}
fn hex(value: &str) -> String {
    format!(
        "{}:{}",
        value.len(),
        value
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}
fn native_metadata(stdout: &str) -> String {
    stdout
        .lines()
        .filter(|line| !line.starts_with("ELEMENT "))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
fn observations(xml: &str) -> Result<String, sdformat::Diagnostic> {
    let sdf = load_str(xml, "gui.sdf", &NoResources)?;
    let Some(gui) = &sdf.worlds[0].gui else {
        return Ok("ERRORS 0\nGUI 0\n".into());
    };
    assert_eq!(gui.source.uri, "gui.sdf");
    assert!(xml.contains(&gui.xml));
    let mut result = format!(
        "ERRORS 0\nGUI 1\nFULLSCREEN {}\nPLUGINS {}\n",
        u8::from(gui.fullscreen),
        gui.plugins.len()
    );
    for plugin in &gui.plugins {
        result += &format!("PLUGIN {} {}\n", hex(&plugin.name), hex(&plugin.filename));
        assert!(gui.xml.contains(&plugin.xml));
    }
    Ok(result)
}
#[test]
fn gui_metadata_matches_native_defaults_boolean_and_ordering_controls() {
    for row in data()["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let result = observations(row["xml"].as_str().unwrap());
        match row["rust_policy"].as_str().unwrap() {
            "accepted" => assert_eq!(
                result.unwrap_or_else(|e| panic!("{name}: {e:?}")),
                native_metadata(row["stdout"].as_str().unwrap()),
                "{name}"
            ),
            policy => {
                let code = match policy {
                    "Missing" => ErrorCode::Missing,
                    "InvalidValue" => ErrorCode::InvalidValue,
                    "Unsupported" => ErrorCode::Unsupported,
                    _ => panic!("unknown policy {policy}"),
                };
                assert_eq!(result.unwrap_err().code, code, "{name}");
            }
        }
    }
}
#[test]
fn original_world_load_gui_values_retain_unchanged_plugin_subtrees() {
    let xml = include_str!("fixtures/world_gui_original.sdf");
    let sdf = load_str(xml, "world_complete.sdf#gui", &NoResources).unwrap();
    assert_eq!(sdf.worlds.len(), 1);
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "default");
    let gui = world.gui.as_ref().unwrap();
    assert!(gui.fullscreen);
    assert_eq!(gui.source.uri, "world_complete.sdf#gui");
    assert_eq!((gui.source.line, gui.source.column), (3, 5));
    assert_eq!(gui.plugins.len(), 2);
    for (i, plugin) in gui.plugins.iter().enumerate() {
        assert_eq!(plugin.name, format!("gui_plugin{}", i + 1));
        assert_eq!(plugin.filename, format!("test/file/gui{}", i + 1));
        assert_eq!(
            plugin.xml,
            format!(
                "<plugin name=\"gui_plugin{}\" filename=\"test/file/gui{}\"/>",
                i + 1,
                i + 1
            )
        );
    }
    assert_eq!(
        observations(xml).unwrap(),
        native_metadata(data()["full_original"]["stdout"].as_str().unwrap())
    );
}
#[test]
fn gui_owned_metadata_and_opaque_plugins_survive_input_drop_and_clone() {
    let original = {
        let xml = String::from(
            "<sdf version='1.12'><world name='w'><gui fullscreen='true'><plugin name='p' filename='f'><v:setting><child>opaque</child></v:setting></plugin></gui></world></sdf>",
        );
        load_str(&xml, "owned-gui.sdf", &NoResources)
            .unwrap()
            .worlds
            .remove(0)
            .gui
            .unwrap()
    };
    let mut copy = original.clone();
    copy.fullscreen = false;
    copy.plugins[0].name = "copy".into();
    copy.plugins[0].xml.clear();
    copy.source.uri.clear();
    copy.xml.clear();
    assert!(original.fullscreen);
    assert_eq!(original.plugins[0].name, "p");
    assert_eq!(original.plugins[0].filename, "f");
    assert_eq!(
        original.plugins[0].xml,
        "<plugin name='p' filename='f'><v:setting><child>opaque</child></v:setting></plugin>"
    );
    assert_eq!(original.source.uri, "owned-gui.sdf");
    assert_eq!((original.source.line, original.source.column), (1, 37));
    assert_eq!(
        original.xml,
        "<gui fullscreen='true'><plugin name='p' filename='f'><v:setting><child>opaque</child></v:setting></plugin></gui>"
    );
    drop(original);
    assert!(!copy.fullscreen);
    assert_eq!(copy.plugins[0].filename, "f");
    assert!(format!("{copy:?}").contains("copy"));
}
#[test]
fn gui_rejects_unimplemented_children_attributes_and_repeated_groups() {
    for (payload, code) in [
        ("<gui><camera name='c'/></gui>", ErrorCode::Unsupported),
        (
            "<gui><fullscreen>true</fullscreen></gui>",
            ErrorCode::Unsupported,
        ),
        ("<gui><unknown/></gui>", ErrorCode::Unsupported),
        ("<gui extra='x'/>", ErrorCode::Unsupported),
        ("<gui/><gui/>", ErrorCode::InvalidValue),
        ("<Gui/>", ErrorCode::Unsupported),
    ] {
        let xml = format!("<sdf version='1.12'><world name='w'>{payload}</world></sdf>");
        assert_eq!(
            load_str(&xml, "gui.sdf", &NoResources).unwrap_err().code,
            code,
            "{payload}"
        );
    }
}
