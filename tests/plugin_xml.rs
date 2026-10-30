/*
 * Copyright (C) 2021 Open Source Robotics Foundation
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
fn children(xml: &str) -> Vec<String> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut depth = 0;
    let mut names = vec![];
    loop {
        match reader.read_event().unwrap() {
            quick_xml::events::Event::Start(e) => {
                if depth == 1 {
                    names.push(String::from_utf8(e.name().as_ref().to_vec()).unwrap());
                }
                depth += 1;
            }
            quick_xml::events::Event::Empty(e) => {
                if depth == 1 {
                    names.push(String::from_utf8(e.name().as_ref().to_vec()).unwrap());
                }
            }
            quick_xml::events::Event::End(_) => depth -= 1,
            quick_xml::events::Event::Eof => break,
            _ => {}
        }
    }
    names
}
#[test]
fn original_plugin_to_element() {
    let plugin = load_plugin_str(
        "<plugin name='my-plugin' filename='filename.so'><an-element/></plugin>",
        "setup.xml",
    )
    .unwrap();
    assert_eq!("my-plugin", plugin.name);
    assert_eq!("filename.so", plugin.filename);
    assert_eq!(1, children(&plugin.xml).len());
    let xml = plugin.to_xml().unwrap();
    assert!(!xml.is_empty());
    let loaded = load_plugin_str(&xml, "loaded.xml").unwrap();
    assert_eq!(plugin.name, loaded.name);
    assert_eq!(plugin.filename, loaded.filename);
    assert_eq!(1, children(&loaded.xml).len());
    assert_eq!("an-element", children(&loaded.xml)[0]);
}
#[test]
fn opaque_payload_and_attributes_remain_exact() {
    for xml in [
        "<plugin filename='f'/>",
        "<plugin filename='f' name='n'><x:config x:a='b'><![CDATA[ a  b ]]></x:config><!--keep--><other/></plugin>",
        "<plugin filename='lib&amp;file' name='a&amp;b' custom='retained'><nested><leaf/></nested></plugin>",
    ] {
        let value = load_plugin_str(xml, "input.xml").unwrap();
        assert_eq!(value.xml, xml);
        let output = value.to_xml().unwrap();
        assert_eq!(output, xml);
        assert_eq!(value.xml, xml);
        let reloaded = load_plugin_str(&output, "output.xml").unwrap();
        assert_eq!(value.name, reloaded.name);
        assert_eq!(value.filename, reloaded.filename);
        assert_eq!(value.xml, reloaded.xml);
    }
    let value = Plugin {
        name: "a&b".into(),
        filename: "lib&file".into(),
        xml: String::new(),
    };
    let loaded = load_plugin_str(&value.to_xml().unwrap(), "x").unwrap();
    assert_eq!(value.name, loaded.name);
    assert_eq!(value.filename, loaded.filename);
    assert!(children(&loaded.xml).is_empty());
}
#[test]
fn unchanged_visual_boundary_and_new_root_diagnostics() {
    let mut value =
        load_plugin_str("<plugin filename='f' name='n'><payload/></plugin>", "x").unwrap();
    value.name = "changed".into();
    let error = value.to_xml().unwrap_err();
    assert_eq!(error.code, ErrorCode::Unsupported);
    assert_eq!(error.source.uri, "<plugin>");
    for xml in ["<plugin name='n'/>", "<plugin filename=''/>"] {
        let error = load_plugin_str(xml, "caller.xml").unwrap_err();
        assert_eq!(error.code, ErrorCode::Missing);
        assert_eq!(error.source.uri, "caller.xml");
        assert_eq!(error.source.line, 1);
        assert_eq!(error.source.column, 1);
    }
    for xml in ["<plugin", "<visual name='v'/>"] {
        assert_eq!(
            load_plugin_str(xml, "caller.xml").unwrap_err().code,
            ErrorCode::Xml
        );
    }
    value.name = "n".into();
    for xml in [
        "<plugin filename='f' name='n'/>\n",
        "<other filename='f' name='n'/>",
    ] {
        value.xml = xml.into();
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
}
fn hex(value: &str) -> String {
    if value.is_empty() {
        "-".into()
    } else {
        value
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}
fn projection(value: &Plugin) -> Vec<String> {
    let children = children(&value.xml);
    let mut fields = vec![
        hex(&value.name),
        hex(&value.filename),
        children.len().to_string(),
    ];
    fields.extend(children.iter().map(|v| hex(v)));
    fields
}
#[test]
fn actual_native_plugin_inputs_and_output_exchange() {
    let data: serde_json::Value = serde_json::from_str(include_str!("plugin_xml.json")).unwrap();
    assert_eq!(13, data["rows"].as_array().unwrap().len());
    for row in data["rows"].as_array().unwrap() {
        let result = load_plugin_str(row["xml"].as_str().unwrap(), "plugin.xml");
        if row.get("gap").is_some() {
            assert_eq!(result.unwrap_err().code, ErrorCode::Missing);
            continue;
        }
        let value = result.unwrap();
        let expected: Vec<String> = serde_json::from_value(row["values"].clone()).unwrap();
        assert_eq!(projection(&value), expected, "{}", row["id"]);
        let before = value.xml.clone();
        let output = value.to_xml().unwrap();
        assert_eq!(output, before);
        assert_eq!(value.xml, before);
        let reloaded = load_plugin_str(&output, "reload.xml").unwrap();
        assert_eq!(projection(&reloaded), expected);
        assert_eq!(reloaded.xml, before);
        if let Some(path) = std::env::var_os("SDF_PLUGIN_XML_EXPORT") {
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(
                std::path::PathBuf::from(path).join(format!("{}.xml", row["id"].as_str().unwrap())),
                output,
            )
            .unwrap();
        }
    }
    let fixture = include_str!("fixtures/plugin_to_element_native.xml");
    let value = load_plugin_str(fixture, "fixture.xml").unwrap();
    assert_eq!(value.name, "my-plugin");
    assert_eq!(value.filename, "filename.so");
    assert_eq!(children(&value.xml), ["an-element"]);
    let visual = load_visual_str(
        "<visual name='v'><plugin name='n' filename='f'/></visual>",
        "x",
    )
    .unwrap();
    let mut changed = visual;
    changed.plugins[0].name = "changed".into();
    assert_eq!(changed.to_xml().unwrap_err().source.uri, "<visual>");
}
