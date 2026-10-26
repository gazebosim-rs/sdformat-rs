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
// Fixed owned values from DOMframe.Construction; graph errors stay unported.
use glam::{DQuat, EulerRot};
use sdformat::Frame;

#[test]
fn partial_dom_frame_construction_values() {
    let mut frame = Frame::default();
    assert!(frame.name.is_empty());
    frame.name = "test_frame".into();
    assert_eq!(frame.name, "test_frame");
    assert!(frame.attached_to.is_empty());
    assert_eq!(frame.pose.transform.translation, [0.; 3]);
    assert_eq!(frame.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(frame.pose.relative_to.is_empty());
    assert!(frame.semantic_relative_to().is_empty());

    frame.attached_to = "attachment".into();
    assert_eq!(frame.attached_to, "attachment");
    frame.pose.transform.translation = [-10., -20., -30.];
    frame.pose.transform.rotation = DQuat::from_euler(
        EulerRot::ZYX,
        std::f64::consts::PI,
        std::f64::consts::PI,
        std::f64::consts::PI,
    )
    .to_array();
    assert_eq!(frame.pose.transform.translation, [-10., -20., -30.]);
    let expected = DQuat::from_euler(
        EulerRot::ZYX,
        std::f64::consts::PI,
        std::f64::consts::PI,
        std::f64::consts::PI,
    );
    assert!(
        (DQuat::from_array(frame.pose.transform.rotation)
            .dot(expected)
            .abs()
            - 1.)
            .abs()
            < 1e-12
    );
    assert_eq!(frame.semantic_relative_to(), "attachment");

    frame.pose.relative_to = "link".into();
    assert_eq!(frame.pose.relative_to, "link");
    assert_eq!(frame.semantic_relative_to(), "link");
}
