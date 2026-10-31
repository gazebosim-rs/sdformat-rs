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

/*
 * Copyright (C) 2019 Open Source Robotics Foundation
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
#[test]
fn audio_device_matches_native_string_observations() {
    let corpus: Value = serde_json::from_str(include_str!("world_audio.json")).unwrap();
    for row in corpus["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let result = load_str(row["xml"].as_str().unwrap(), "audio.sdf", &NoResources);
        match row["rust_policy"].as_str().unwrap() {
            "accepted" => {
                let sdf = result.unwrap_or_else(|error| panic!("{name}: {error:?}"));
                let world = &sdf.worlds[0];
                let bytes = world.audio_device.as_bytes();
                let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
                assert_eq!(
                    row["stdout"].as_str().unwrap(),
                    format!("ERRORS 0\nDEVICE {}:{hex}\n", bytes.len()),
                    "{name}"
                );
                assert_eq!(world.source.uri, "audio.sdf");
                assert_eq!(world.name, "w");
                assert!(world.extensions.is_empty());
                assert!(world.models.is_empty());
                assert!(world.lights.is_empty());
            }
            "Missing" => assert_eq!(result.unwrap_err().code, ErrorCode::Missing, "{name}"),
            "InvalidValue" => {
                assert_eq!(result.unwrap_err().code, ErrorCode::InvalidValue, "{name}")
            }
            policy => panic!("unknown policy {policy}"),
        }
    }
}

#[test]
fn unsupported_audio_content_does_not_become_opaque_metadata() {
    for payload in [
        "<audio extra='x'/>",
        "<audio><unknown/></audio>",
        "<audio><device extra='x'>default</device></audio>",
        "<audio><device><unknown/></device></audio>",
        "<Audio/>",
    ] {
        let xml = format!("<sdf version='1.12'><world name='w'>{payload}</world></sdf>");
        assert_eq!(
            load_str(&xml, "audio.sdf", &NoResources).unwrap_err().code,
            ErrorCode::Unsupported,
            "{payload}"
        );
    }
}

#[test]
fn loaded_device_is_owned_metadata() {
    let mut sdf = {
        let xml = String::from(
            "<sdf version='1.12'><world name='w'><audio><device>/dev/audio</device></audio></world></sdf>",
        );
        load_str(&xml, "audio.sdf", &NoResources).unwrap()
    };
    assert_eq!(sdf.worlds[0].audio_device, "/dev/audio");
    sdf.worlds[0].audio_device = "another device".into();
    assert_eq!(sdf.worlds[0].audio_device, "another device");
    assert_eq!(sdf.worlds[0].source.uri, "audio.sdf");
}

#[test]
fn original_dom_world_load_audio_assertion_on_unmodified_subtree() {
    let xml = include_str!("fixtures/world_audio_original.sdf");
    let sdf = load_str(xml, "world_complete.sdf#audio", &NoResources).unwrap();
    assert_eq!(sdf.worlds.len(), 1);
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "default");
    assert_eq!(world.audio_device, "/dev/audio");
    assert_eq!(world.source.uri, "world_complete.sdf#audio");
    assert!(world.models.is_empty());
    assert!(world.extensions.is_empty());
}
