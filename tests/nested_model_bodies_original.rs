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
//
// Partial assertion port: NestedFrameSemantics.buildFrameAttachedToGraph_Model.

use sdformat::{ErrorCode, NoResources, Sdf, load_str};

fn body(sdf: &Sdf, model: &str, frame: &str) -> String {
    let name = sdf.resolve_name(model, frame).unwrap();
    let resolved = sdf.attached_body(&name).unwrap();
    resolved
        .strip_prefix(&format!("{model}::"))
        .unwrap_or(resolved)
        .to_string()
}

#[test]
fn original_nested_model_attachment_bodies() {
    let sdf = load_str(
        include_str!("fixtures/model_nested_frame_attached_to.sdf"),
        "model_nested_frame_attached_to.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().expect("original root.Model()");
    assert_eq!(model.name, "nested_model_frame_attached_to");
    let scope = &model.scoped_name;
    for (frame, expected) in [
        ("__model__", "L"),
        ("L", "L"),
        ("M1", "M1::L"),
        ("M1::__model__", "M1::L"),
        ("M1::L", "M1::L"),
        ("M1::M2", "M1::M2::L"),
        ("M1::M2::L", "M1::M2::L"),
        ("M1::F", "M1::M2::L"),
        ("F0", "M1::L"),
        ("F1", "M1::L"),
        ("F2", "M1::L"),
        ("F3", "M1::M2::L"),
        ("F4", "M1::M2::L"),
    ] {
        assert_eq!(body(&sdf, scope, frame), expected, "{frame}");
    }
    let err = sdf.attached_body(&format!("{scope}::invalid")).unwrap_err();
    assert_eq!(err.code, ErrorCode::UnknownReference);
}
