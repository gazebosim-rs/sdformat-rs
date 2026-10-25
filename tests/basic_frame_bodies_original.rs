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
// Partial ports: FrameSemantics.buildFrameAttachedToGraph_Model and _World.

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
fn original_model_frame_attachment_bodies() {
    let sdf = load_str(
        include_str!("fixtures/model_frame_attached_to.sdf"),
        "model_frame_attached_to.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().expect("original root.Model()");
    let scope = &model.scoped_name;
    for frame in ["L", "__model__", "F00", "F0", "F1", "F2"] {
        body(&sdf, scope, frame, "L");
    }
    assert_eq!(
        sdf.attached_body(&format!("{scope}::invalid"))
            .unwrap_err()
            .code,
        ErrorCode::UnknownReference
    );
}

#[test]
fn original_world_frame_attachment_bodies() {
    let sdf = load_str(
        include_str!("fixtures/world_frame_attached_to.sdf"),
        "world_frame_attached_to.sdf",
        &NoResources,
    )
    .unwrap();
    let world = sdf.worlds.first().expect("original WorldByIndex(0)");
    let scope = &world.name;
    for frame in ["world", "world_frame", "F0", "F1"] {
        body(&sdf, scope, frame, "world");
    }
    for frame in ["M1", "F2"] {
        body(&sdf, scope, frame, "M1::L");
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
    for frame in ["L", "__model__", "F0"] {
        body(&sdf, &model.scoped_name, frame, "L");
    }
}
