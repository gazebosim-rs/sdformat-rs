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
// Pinned src/Plugin_TEST.cc DOMPlugin CopyConstructor, CopyAssigmentOperator,
// MoveAssignmentConstructor and CopyAssignmentAfterMove.
use sdformat::Plugin;

fn new_plugin(name: &str, filename: &str) -> Plugin {
    Plugin {
        name: name.into(),
        filename: filename.into(),
        xml: String::new(),
    }
}

#[test]
fn original_dom_plugin_copy_constructor() {
    let plugin = new_plugin("pluginname", "filename");
    let plugin2 = plugin.clone();
    assert_eq!(plugin2.name, "pluginname");
    assert_eq!(plugin2.filename, "filename");
    assert_eq!(plugin.name, "pluginname");
    assert_eq!(plugin.filename, "filename");
}

#[test]
fn original_dom_plugin_copy_assigment_operator() {
    let plugin = new_plugin("pluginname", "filename");
    let mut plugin2 = new_plugin("", "");
    plugin2.clone_from(&plugin);
    assert_eq!(plugin2.name, "pluginname");
    assert_eq!(plugin2.filename, "filename");
    assert_eq!(plugin.name, "pluginname");
    assert_eq!(plugin.filename, "filename");
}

#[test]
fn original_dom_plugin_move_assignment_constructor() {
    let plugin = new_plugin("pluginname", "filename");
    let mut plugin2 = new_plugin("", "");
    assert!(plugin2.name.is_empty());
    plugin2 = plugin;
    assert_eq!(plugin2.name, "pluginname");
    assert_eq!(plugin2.filename, "filename");
}

#[test]
fn original_dom_plugin_copy_assignment_after_move() {
    let mut plugin = new_plugin("pluginname", "filename");
    let mut plugin2 = new_plugin("pluginname2", "filename2");
    std::mem::swap(&mut plugin, &mut plugin2);
    assert_eq!(plugin2.name, "pluginname");
    assert_eq!(plugin2.filename, "filename");
    assert_eq!(plugin.name, "pluginname2");
    assert_eq!(plugin.filename, "filename2");
}
