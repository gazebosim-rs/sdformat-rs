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
// Partial port: NestedFrameSemantics.buildFrameAttachedToGraph_World.

use sdformat::{ErrorCode, NoResources, Sdf, load_str};

fn body(sdf: &Sdf, scope: &str, local: &str, expected: &str) {
    let frame = sdf.resolve_name(scope, local).unwrap();
    let resolved = sdf.attached_body(&frame).unwrap();
    let expected = if expected == "world" {
        scope.to_string()
    } else {
        format!("{scope}::{expected}")
    };
    assert_eq!(resolved, expected, "{local}");
}

#[test]
fn original_nested_world_frame_attachment_bodies() {
    let sdf = load_str(
        include_str!("fixtures/world_nested_frame_attached_to.sdf"),
        "world_nested_frame_attached_to.sdf",
        &NoResources,
    )
    .unwrap();
    let world = sdf.worlds.first().expect("original WorldByIndex(0)");
    let scope = &world.name;
    for frame in ["world", "world_frame", "F0"] {
        body(&sdf, scope, frame, "world");
    }
    for frame in ["M1", "M1::__model__", "M1::L", "F1", "F2"] {
        body(&sdf, scope, frame, "M1::L");
    }
    for frame in [
        "M1::M2",
        "M1::M2::__model__",
        "M1::M2::L",
        "M1::F0",
        "F3",
        "F4",
        "F5",
        "F6",
    ] {
        body(&sdf, scope, frame, "M1::M2::L");
    }
    assert_eq!(
        sdf.attached_body(&format!("{scope}::invalid"))
            .unwrap_err()
            .code,
        ErrorCode::UnknownReference
    );

    let model = world
        .models
        .first()
        .expect("original world.ModelByIndex(0)");
    let model_scope = &model.scoped_name;
    for frame in ["__model__", "L"] {
        body(&sdf, model_scope, frame, "L");
    }
    for frame in ["M2", "M2::__model__", "M2::L", "F0"] {
        body(&sdf, model_scope, frame, "M2::L");
    }
    // The public name resolver treats `world` as an enclosing-world alias.
    // Native modelGraph resolution rejects it, so that graph-only assertion
    // remains unported rather than pretending a prefixed missing name is equal.
    assert_eq!(sdf.resolve_name(model_scope, "world").unwrap(), world.name);
    for frame in ["world_frame", "F1", "F2", "F3", "F4", "F5", "F6"] {
        assert_eq!(
            sdf.resolve_name(model_scope, frame).unwrap_err().code,
            ErrorCode::UnknownReference,
            "{frame}"
        );
    }
}
