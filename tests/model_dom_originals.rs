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
// Original assertions: model_dom.cc DOMRoot.LoadDoublePendulum and NestedModel.
use sdformat::{NoResources, Transform, load_str};

fn pose(actual: Transform, expected: [f64; 6]) {
    let q = glam::DQuat::from_euler(glam::EulerRot::ZYX, expected[5], expected[4], expected[3]);
    for (a, b) in actual
        .translation
        .into_iter()
        .zip(expected[..3].iter().copied())
    {
        assert!((a - b).abs() <= 1e-3);
    }
    for (a, b) in actual.rotation.into_iter().zip(q.to_array()) {
        assert!((a - b).abs() <= 1e-3);
    }
}

#[test]
// Retain the original index-zero presence assertions as well as count assertions.
#[allow(clippy::unnecessary_first_then_check)]
fn original_root_load_double_pendulum() {
    // Native ValidateGraphs succeeds for this unchanged fixture. Loading builds
    // and validates both frame graphs before returning the owned DOM.
    let checked = load_str(
        include_str!("fixtures/double_pendulum.sdf"),
        "double_pendulum.sdf",
        &NoResources,
    );
    assert!(checked.is_ok());
    let sdf = checked.unwrap();
    let model = sdf.models.first().unwrap();
    assert_eq!(model.name, "double_pendulum_with_base");
    assert_eq!(model.links.len(), 3);
    assert!(model.links.first().is_some());
    assert!(model.links.get(1).is_some());
    assert!(model.links.get(2).is_some());
    assert!(model.links.get(3).is_none());
    pose(model.pose.transform, [1., 0., 0., 0., 0., 0.]);
    assert_eq!(model.pose.relative_to, "");
    assert!(model.link_by_name("base").is_some());
    assert!(model.link_by_name("upper_link").is_some());
    assert!(model.link_by_name("lower_link").is_some());
    assert_eq!(model.joints.len(), 2);
    assert!(model.joints.first().is_some());
    assert!(model.joints.get(1).is_some());
    assert!(model.joints.get(2).is_none());
    assert!(model.joints.iter().any(|joint| joint.name == "upper_joint"));
    assert!(model.joints.iter().any(|joint| joint.name == "lower_joint"));
}

#[test]
// Retain the original index-zero presence assertions as well as count assertions.
#[allow(clippy::unnecessary_first_then_check)]
fn original_root_nested_model() {
    let checked = load_str(
        include_str!("fixtures/nested_model.sdf"),
        "nested_model.sdf",
        &NoResources,
    );
    assert!(checked.is_ok());
    let sdf = checked.unwrap();
    let model = sdf.models.first().unwrap();
    assert_eq!(model.name, "top_level_model");
    assert_eq!(model.links.len(), 2);
    assert!(model.links.first().is_some());
    assert!(model.links.get(1).is_some());
    assert!(model.links.get(2).is_none());
    pose(model.pose.transform, [0.; 6]);
    assert_eq!(model.pose.relative_to, "");
    assert!(model.link_by_name("parent").is_some());
    assert!(model.link_by_name("child").is_some());
    assert_eq!(model.joints.len(), 1);
    assert!(model.joints.first().is_some());
    assert!(model.joints.get(1).is_none());
    assert!(
        model
            .joints
            .iter()
            .any(|joint| joint.name == "top_level_joint")
    );
    assert_eq!(model.models.len(), 1);
    let nested = model.models.first().unwrap();
    assert!(model.models.get(1).is_none());
    assert!(model.model_by_name("nested_model").is_some());
    assert!(std::ptr::eq(
        nested,
        model.model_by_name("nested_model").unwrap()
    ));
    assert_eq!(nested.name, "nested_model");
    assert_eq!(nested.links.len(), 1);
    assert!(nested.links.first().is_some());
    assert!(nested.links.get(1).is_none());
    assert!(nested.link_by_name("nested_link01").is_some());
    assert_eq!(nested.joints.len(), 0);
    assert_eq!(nested.frames.len(), 0);
    assert!(model.link_by_name("nested_model::nested_link01").is_some());
    let nested_link = model.link_by_name("nested_model::nested_link01");
    assert!(nested_link.is_some());
    assert!(model.model_by_name("nested_nested_model").is_none());
    assert!(model.model_by_name("nested_nested_model").is_none());
    assert!(
        model
            .model_by_name("nested_model::nested_nested_model")
            .is_some()
    );
    let deepest = model
        .model_by_name("nested_model::nested_nested_model")
        .unwrap();
    assert!(deepest.link_by_name("nested_nested_link01").is_some());
}
