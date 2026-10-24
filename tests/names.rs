// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
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

use sdformat::*;

// Original DOMUtils.ReservedNames table, observed through actual DOM loading.
// The name validator is private; these inputs exercise it without another invalid field.
#[test]
fn reserved_names_original_assertions_through_dom() {
    for (name, reserved) in [
        ("model", false),
        ("link", false),
        ("joint", false),
        ("frame", false),
        ("collision", false),
        ("visual", false),
        ("not_reserved", false),
        ("_", false),
        ("__", false),
        ("___", false),
        ("____", true),
        ("world", true),
        ("__model__", true),
        ("__world__", true),
        ("__anything__", true),
    ] {
        let xml =
            format!("<sdf version='1.12'><model name='{name}'><link name='l'/></model></sdf>");
        let result = load_str(&xml, "reserved-names", &NoResources);
        assert_eq!(result.is_err(), reserved, "{name}");
        if let Err(error) = result {
            assert_eq!(error.code, ErrorCode::InvalidValue);
        }
    }
}
#[test]
fn native_reserved_names_apply_to_every_supported_named_entity() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("names.json")).unwrap();
    for case in cases.as_array().unwrap() {
        assert_eq!(
            load_str(
                case["xml"].as_str().unwrap(),
                case["id"].as_str().unwrap(),
                &NoResources
            )
            .is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{case}"
        );
    }
}
