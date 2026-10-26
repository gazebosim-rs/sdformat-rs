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
// Exact inline SDF from test/integration/joint_axis_dom.cc ParseMimic.
use sdformat::{NoResources, load_str};

#[test]
fn original_dom_joint_axis_parse_mimic() {
    let sdf = load_str(
        include_str!("fixtures/joint_axis_mimic.sdf"),
        "joint_axis_mimic.sdf",
        &NoResources,
    )
    .expect("native Root.LoadSdfString has no errors");
    let model = sdf.models.first().expect("native Root.Model is present");
    let joint = model
        .joint_by_name("follow_joint")
        .expect("native JointByName succeeds");
    let axis = joint.axis(0).expect("native Axis(0) is present");
    let mimic = axis.mimic.as_ref().expect("native Mimic is present");
    assert_eq!(mimic.joint, "source_joint");
    assert_eq!(mimic.axis, "axis");
    assert_eq!(mimic.multiplier, 4.);
    assert_eq!(mimic.offset, 2.);
    assert_eq!(mimic.reference, 3.);
}
