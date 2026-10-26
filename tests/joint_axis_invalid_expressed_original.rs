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
// First error only from DOMJointAxis.InvalidExpressedIn; native emits two.
use sdformat::{ErrorCode, NoResources, load_str};

#[test]
fn invalid_expressed_in_retains_native_first_diagnostic() {
    let error = load_str(
        include_str!("fixtures/joint_axis_invalid_expressed_in.sdf"),
        "joint_axis_invalid_expressed_in.sdf",
        &NoResources,
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::JointAxisExpressedInInvalid);
    assert_eq!(
        error.message,
        "axis xyz expressed-in frame with name[invalid] specified by joint with name[joint] not found in model with name[joint_axis_invalid_expressed_in]."
    );
}
