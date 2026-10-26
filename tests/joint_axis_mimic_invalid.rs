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
// First-error controls from three pinned DOMJointAxis mimic originals.
use sdformat::{ErrorCode, NoResources, load_str};

fn first_error(xml: &str) -> sdformat::Diagnostic {
    load_str(xml, "joint_axis_mimic_invalid.sdf", &NoResources).unwrap_err()
}

#[test]
fn self_mimic_rejects_the_first_axis() {
    let error = first_error(include_str!("fixtures/joint_axis_mimic_self.sdf"));
    assert_eq!(error.code, ErrorCode::JointAxisMimicInvalid);
    assert_eq!(
        error.message,
        "Axis with name [axis] in joint with name [self_mimic] cannot mimic itself."
    );
}

#[test]
fn missing_leader_joint_rejects_the_first_axis() {
    let error = first_error(include_str!("fixtures/joint_axis_mimic_missing_joint.sdf"));
    assert_eq!(error.code, ErrorCode::JointAxisMimicInvalid);
    assert_eq!(
        error.message,
        "A joint with name[invalid] specified by an axis mimic in joint with name[joint] not found in model with name[test]."
    );
}

#[test]
fn invalid_leader_axis_rejects_the_first_axis() {
    let error = first_error(include_str!("fixtures/joint_axis_mimic_invalid_axis.sdf"));
    assert_eq!(error.code, ErrorCode::JointAxisMimicInvalid);
    assert_eq!(
        error.message,
        "Axis with name [axis] in joint with name [follower] specified an invalid leader axis name [invalid]."
    );
}

#[test]
fn absent_leader_axis_rejects_the_follower() {
    let xml = include_str!("fixtures/joint_axis_mimic.sdf").replace(
        "<mimic joint='source_joint'>",
        "<mimic joint='source_joint' axis='axis2'>",
    );
    let error = first_error(&xml);
    assert_eq!(error.code, ErrorCode::JointAxisMimicInvalid);
    assert_eq!(
        error.message,
        "Axis with name [axis] in joint with name [follow_joint] specified a leader axis name [axis2] that is not found in the leader joint with name [source_joint]."
    );
}
