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
// Observable values from five pinned DOMJoint copy and move originals.
use sdformat::{Joint, NoResources, load_str};

fn joint(name: &str, axis0: &str, axis1: &str) -> Joint {
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='{name}' type='revolute'><parent>a</parent><child>b</child><axis><xyz>{axis0}</xyz></axis><axis2><xyz>{axis1}</xyz></axis2></joint></model></sdf>"
    );
    load_str(&xml, "joint-ownership.sdf", &NoResources)
        .unwrap()
        .models[0]
        .joints
        .remove(0)
}

fn values(joint: &Joint, name: &str, axis0: [f64; 3], axis1: [f64; 3]) {
    assert_eq!(joint.name, name);
    assert_eq!(joint.axis(0).expect("Axis(0)").xyz, axis0);
    assert_eq!(joint.axis(1).expect("Axis(1)").xyz, axis1);
}

#[test]
fn original_dom_joint_move_constructor() {
    let source = joint("test_joint", "1 0 0", "0 1 0");
    values(&source, "test_joint", [1., 0., 0.], [0., 1., 0.]);
    let moved = source;
    values(&moved, "test_joint", [1., 0., 0.], [0., 1., 0.]);
}

#[test]
fn original_dom_joint_copy_constructor() {
    let source = joint("test_joint", "1 0 0", "0 1 0");
    let copy = source.clone();
    values(&source, "test_joint", [1., 0., 0.], [0., 1., 0.]);
    values(&copy, "test_joint", [1., 0., 0.], [0., 1., 0.]);
}

#[test]
fn original_dom_joint_move_assignment() {
    let source = joint("test_joint", "1 0 0", "0 1 0");
    let mut destination = joint("other", "0 0 1", "-1 0 0");
    values(&destination, "other", [0., 0., 1.], [-1., 0., 0.]);
    destination = source;
    values(&destination, "test_joint", [1., 0., 0.], [0., 1., 0.]);
}

#[test]
fn original_dom_joint_copy_assignment() {
    let source = joint("test_joint", "1 0 0", "0 1 0");
    let mut destination = joint("other", "0 0 1", "-1 0 0");
    values(&destination, "other", [0., 0., 1.], [-1., 0., 0.]);
    destination.clone_from(&source);
    values(&source, "test_joint", [1., 0., 0.], [0., 1., 0.]);
    values(&destination, "test_joint", [1., 0., 0.], [0., 1., 0.]);
}

#[test]
fn original_dom_joint_copy_assignment_after_move() {
    let mut first = joint("test_joint1", "1 0 0", "0 1 0");
    let mut second = joint("test_joint2", "0 0 1", "-1 0 0");
    let temporary = std::mem::replace(&mut first, second.clone());
    second = temporary;
    values(&first, "test_joint2", [0., 0., 1.], [-1., 0., 0.]);
    values(&second, "test_joint1", [1., 0., 0.], [0., 1., 0.]);
}
