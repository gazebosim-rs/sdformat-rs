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
/*
 * Copyright (C) 2017 Open Source Robotics Foundation
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
// Pinned Model_TEST.cc:703, World_TEST.cc:879 and Visual_TEST.cc:337.
use sdformat::{NoResources, Plugin, load_str};

fn original_plugin_observations(plugins: &mut Vec<Plugin>) {
    assert!(plugins.is_empty());
    let mut plugin = Plugin {
        name: "name1".into(),
        filename: "filename1".into(),
        xml: String::new(),
    };
    plugins.push(plugin.clone());
    assert_eq!(plugins.len(), 1);

    plugin.name = "name2".into();
    plugins.push(plugin);
    assert_eq!(plugins.len(), 2);
    assert_eq!(plugins[0].name, "name1");
    assert_eq!(plugins[1].name, "name2");

    plugins.clear();
    assert!(plugins.is_empty());
}

#[test]
fn original_dom_model_plugins() {
    let mut sdf = load_str(
        "<sdf version='1.12'><model name='m'><link name='l'/></model></sdf>",
        "model-plugins.sdf",
        &NoResources,
    )
    .unwrap();
    original_plugin_observations(&mut sdf.models[0].plugins);
}

#[test]
fn original_dom_world_plugins() {
    let mut sdf = load_str(
        "<sdf version='1.12'><world name='w'/></sdf>",
        "world-plugins.sdf",
        &NoResources,
    )
    .unwrap();
    original_plugin_observations(&mut sdf.worlds[0].plugins);
}

#[test]
fn original_dom_visual_plugins() {
    let mut sdf = load_str(
        "<sdf version='1.12'><model name='m'><link name='l'><visual name='v'/></link></model></sdf>",
        "visual-plugins.sdf",
        &NoResources,
    )
    .unwrap();
    original_plugin_observations(&mut sdf.models[0].links[0].visuals[0].plugins);
}
