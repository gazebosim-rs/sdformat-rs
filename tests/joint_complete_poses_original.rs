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
use glam::{DQuat, EulerRot};
use sdformat::{NoResources, load_str};

#[test]
fn original_dom_joint_complete() {
    let result = load_str(
        include_str!("fixtures/joint_complete.sdf"),
        "joint_complete.sdf",
        &NoResources,
    );
    assert!(result.is_ok(), "{:?}", result.as_ref().err());
    let sdf = result.unwrap();
    let model = sdf.models.first().expect("Root.Model() is present");
    let expected = [
        [1., 0., 0., 0., 0., 0.],
        [0., 1., 0., 0., 0., 0.],
        [0., 0., 1., 0., 0., 0.],
        [0., 0., 0., 1., 0., 0.],
        [0., 0., 0., 0., 1., 0.],
        [0., 0., 0., 0., 0., 1.],
        [2., 0., 0., 0., 0., 0.],
        [0., 2., 0., 0., 0., 0.],
        [0., 0., 2., 0., 0., 0.],
    ];
    for (index, [x, y, z, roll, pitch, yaw]) in expected.into_iter().enumerate() {
        let joint = model.joints.get(index).expect("JointByIndex(i) is present");
        let actual = joint.pose.transform;
        let rotation = DQuat::from_euler(EulerRot::ZYX, yaw, pitch, roll).to_array();
        for (actual, expected) in actual
            .translation
            .into_iter()
            .chain(actual.rotation)
            .zip([x, y, z].into_iter().chain(rotation))
        {
            assert!(
                (actual - expected).abs() <= 1e-3,
                "joint{index}: {actual} versus {expected}"
            );
        }
    }
    let joint = model
        .joint_by_name("screw_joint")
        .expect("JointByName(screw_joint) is present");
    // The native loaded Element presence maps to this exact loaded XML child.
    let loaded_joint = model.element().unwrap().children().iter().find(|child| {
        child.tag() == "joint" && child.name_attribute() == Some(joint.name.as_str())
    });
    assert!(loaded_joint.is_some());
    assert!(
        joint
            .legacy_thread_pitch()
            .to_bits()
            .abs_diff(20_f64.to_bits())
            <= 4
    );
}
