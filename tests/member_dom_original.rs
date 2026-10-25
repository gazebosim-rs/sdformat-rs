/*
 * Copyright 2018 Open Source Robotics Foundation
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
// Original assertions: model_dom.cc DOMRoot.MultiNestedModel.
use sdformat::{ErrorCode, NoResources, load_str};

#[test]
// Preserve upstream index-zero assertions separately from the count checks.
#[allow(clippy::unnecessary_first_then_check)]
fn original_root_multi_nested_model() {
    let checked = load_str(
        include_str!("fixtures/model_multi_nested_model.sdf"),
        "model_multi_nested_model.sdf",
        &NoResources,
    );
    assert!(checked.is_ok());
    let sdf = checked.unwrap();
    let outer = sdf.models.first().unwrap();
    assert_eq!(outer.name, "outer_model");
    assert_eq!(outer.links.len(), 1);
    assert!(outer.links.first().is_some());
    assert!(outer.links.get(1).is_none());
    assert!(outer.link_by_name("outer_link").is_some());
    assert_eq!(outer.joints.len(), 1);
    assert!(outer.joint_by_name("outer_joint").is_some());
    assert_eq!(outer.frames.len(), 1);
    assert!(outer.frame_by_name("outer_frame").is_some());
    assert_eq!(outer.models.len(), 1);
    assert!(outer.models.first().is_some());
    assert!(outer.models.get(1).is_none());
    assert!(outer.model_by_name("mid_model").is_some());
    let mid = outer.models.first().unwrap();
    assert_eq!(mid.name, "mid_model");
    assert_eq!(mid.links.len(), 1);
    assert!(mid.links.first().is_some());
    assert!(mid.links.get(1).is_none());
    assert!(mid.link_by_name("mid_link").is_some());
    assert_eq!(mid.joints.len(), 1);
    assert!(mid.joint_by_name("mid_joint").is_some());
    assert_eq!(mid.frames.len(), 1);
    assert!(mid.frame_by_name("mid_frame").is_some());
    assert_eq!(mid.models.len(), 1);
    assert!(mid.models.first().is_some());
    assert!(mid.models.get(1).is_none());
    assert!(mid.model_by_name("inner_model").is_some());
    let inner = mid.models.first().unwrap();
    assert_eq!(inner.name, "inner_model");
    assert_eq!(inner.links.len(), 1);
    assert!(inner.links.first().is_some());
    assert!(inner.links.get(1).is_none());
    assert!(inner.link_by_name("inner_link").is_some());
    assert_eq!(inner.joints.len(), 1);
    assert!(inner.joint_by_name("inner_joint").is_some());
    assert_eq!(inner.frames.len(), 1);
    assert!(inner.frame_by_name("inner_frame").is_some());
    assert_eq!(inner.models.len(), 0);
    let model_name = "mid_model::inner_model";
    assert!(outer.model_by_name(model_name).is_some());
    assert!(std::ptr::eq(
        inner,
        outer.model_by_name(model_name).unwrap()
    ));
    let link_name = "mid_model::inner_model::inner_link";
    assert!(outer.link_by_name(link_name).is_some());
    assert!(std::ptr::eq(
        inner.links.first().unwrap(),
        outer.link_by_name(link_name).unwrap()
    ));
    assert!(outer.link_by_name(link_name).is_some());
    let joint_name = "mid_model::inner_model::inner_joint";
    assert!(outer.joint_by_name(joint_name).is_some());
    assert!(std::ptr::eq(
        inner.joints.first().unwrap(),
        outer.joint_by_name(joint_name).unwrap()
    ));
    assert!(outer.joint_by_name(joint_name).is_some());
    let frame_name = "mid_model::inner_model::inner_frame";
    assert!(outer.frame_by_name(frame_name).is_some());
    assert!(std::ptr::eq(
        inner.frames.first().unwrap(),
        outer.frame_by_name(frame_name).unwrap()
    ));
    assert!(outer.frame_by_name(frame_name).is_some());
    // On this unchanged validated graph, successful attachment lookup preserves
    // the native NameExistsInFrameAttachedToGraph observations at the same names.
    for name in ["outer_link", "outer_joint", "outer_frame", "mid_model"] {
        assert!(sdf.attached_body(&format!("outer_model::{name}")).is_ok());
    }
    assert_eq!(
        sdf.attached_body("outer_model::mid_link").unwrap_err().code,
        ErrorCode::UnknownReference
    );
    assert!(
        sdf.attached_body("outer_model::mid_model::mid_link")
            .is_ok()
    );
    assert!(
        sdf.attached_body("outer_model::mid_model::inner_model::inner_joint")
            .is_ok()
    );
}
