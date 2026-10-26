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
// Fixed observable values from DOMLink.AddLight.
use sdformat::{Light, NoResources, load_str};

#[test]
fn original_dom_link_add_light() {
    let mut link = load_str(
        "<sdf version='1.12'><model name='parent'><link name='link'/></model></sdf>",
        "link-add-light.sdf",
        &NoResources,
    )
    .unwrap()
    .models
    .remove(0)
    .links
    .remove(0);
    assert_eq!(link.lights.len(), 0);

    let light = Light {
        name: "light1".into(),
    };
    assert!(link.add_light_unique(light.clone()));
    assert_eq!(link.lights.len(), 1);
    assert!(!link.add_light_unique(light.clone()));
    assert_eq!(link.lights.len(), 1);

    link.clear_lights();
    assert_eq!(link.lights.len(), 0);
    assert!(link.add_light_unique(light.clone()));
    assert_eq!(link.lights.len(), 1);
    let light_from_link = link.lights.first().expect("LightByIndex(0)");
    assert_eq!(light_from_link.name, light.name);
}
