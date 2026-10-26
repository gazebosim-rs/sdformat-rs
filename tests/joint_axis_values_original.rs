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
// Pinned src/JointAxis_TEST.cc five copy and move XYZ value originals.
use sdformat::{Axis, NoResources, load_str};

fn axis(xyz: &str) -> Axis {
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child><axis><xyz>{xyz}</xyz></axis></joint></model></sdf>"
    );
    load_str(&xml, "joint-axis-value.sdf", &NoResources)
        .unwrap()
        .models[0]
        .joints[0]
        .axis
        .clone()
        .unwrap()
}

#[test]
fn original_dom_joint_axis_copy_constructor() {
    let joint_axis = axis("0 1 0");
    assert_eq!(joint_axis.xyz, [0., 1., 0.]);
    let copy = joint_axis.clone();
    assert_eq!(joint_axis.xyz, copy.xyz);
}

#[test]
fn original_dom_joint_axis_assignment_operator() {
    let joint_axis = axis("0 1 0");
    assert_eq!(joint_axis.xyz, [0., 1., 0.]);
    let mut copy = axis("0 0 1");
    copy.clone_from(&joint_axis);
    assert_eq!(joint_axis.xyz, copy.xyz);
}

#[test]
fn original_dom_joint_axis_move_constructor() {
    let expected = [0., 1., 0.];
    let joint_axis = axis("0 1 0");
    assert_eq!(joint_axis.xyz, expected);
    let moved = joint_axis;
    assert_eq!(expected, moved.xyz);
}

#[test]
fn original_dom_joint_axis_move_assignment_operator() {
    let expected = [0., 1., 0.];
    let joint_axis = axis("0 1 0");
    assert_eq!(joint_axis.xyz, expected);
    let mut moved = axis("0 0 1");
    assert_eq!(moved.xyz, [0., 0., 1.]);
    moved = joint_axis;
    assert_eq!(expected, moved.xyz);
}

#[test]
fn original_dom_joint_axis_copy_assignment_after_move() {
    let axis1 = [0., 1., 0.];
    let axis2 = [1., 0., 0.];
    let mut joint_axis1 = axis("0 1 0");
    let mut joint_axis2 = axis("1 0 0");
    assert_eq!(joint_axis1.xyz, axis1);
    assert_eq!(joint_axis2.xyz, axis2);
    std::mem::swap(&mut joint_axis1, &mut joint_axis2);
    assert_eq!(axis2, joint_axis1.xyz);
    assert_eq!(axis1, joint_axis2.xyz);
}
