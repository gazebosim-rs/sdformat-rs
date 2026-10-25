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
// Partial assertion port: NestedFrameSemantics.InvalidAttachedToScope.

use sdformat::{ErrorCode, NoResources, load_str};

const FIXTURE: &str = include_str!("fixtures/world_frame_invalid_attached_to_scope.sdf");

#[test]
fn original_invalid_world_frame_attachment_scope() {
    let err = load_str(
        FIXTURE,
        "world_frame_invalid_attached_to_scope.sdf",
        &NoResources,
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::UnknownReference);
    assert_eq!(
        err.message,
        "attached_to name[M2::M1::L1] specified by frame with name[F] \
         does not match a model or frame name in world with \
         name[world_frame_invalid_attached_to_scope]."
    );
}

#[test]
fn native_world_frame_attachment_controls() {
    for target in ["M1", "M2::M1::L2"] {
        let xml = FIXTURE.replace("M2::M1::L1", target);
        assert!(
            load_str(&xml, target, &NoResources).is_ok(),
            "native accepts {target}"
        );
    }
    let xml = FIXTURE.replace("M2::M1::L1", "M9::L1");
    let err = load_str(&xml, "invalid-top", &NoResources).unwrap_err();
    assert_eq!(err.code, ErrorCode::UnknownReference);
    assert_eq!(
        err.message,
        "attached_to name[M9::L1] specified by frame with name[F] \
         does not match a model or frame name in world with \
         name[world_frame_invalid_attached_to_scope]."
    );
}
