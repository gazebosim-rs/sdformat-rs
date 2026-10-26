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
// Exact final name observations from five pinned DOMPhysics originals.
use sdformat::Physics;

fn named_physics(name: &str) -> Physics {
    Physics {
        name: name.into(),
        ..Physics::default()
    }
}

#[test]
fn original_dom_physics_copy_constructor() {
    let physics = named_physics("test_physics");
    let physics2 = physics.clone();
    assert_eq!(physics2.name, "test_physics");
}

#[test]
fn original_dom_physics_copy_assignment_operator() {
    let physics = named_physics("test_physics");
    let mut physics2 = Physics::default();
    physics2.clone_from(&physics);
    assert_eq!(physics2.name, "test_physics");
}

#[test]
fn original_dom_physics_move_constructor() {
    let physics = named_physics("test_physics");
    // The pinned test named MoveConstructor invokes the copy constructor.
    let physics2 = physics.clone();
    assert_eq!(physics2.name, "test_physics");
}

#[test]
fn original_dom_physics_move_assignment_operator() {
    let physics = named_physics("test_physics");
    let mut physics2 = Physics::default();
    let previous = std::mem::replace(&mut physics2, physics);
    assert_eq!(previous, Physics::default()); // Supplemental replacement control.
    assert_eq!(physics2.name, "test_physics");
}

#[test]
fn original_dom_physics_copy_assignment_after_move() {
    let physics1 = named_physics("physics1");
    let physics2 = named_physics("physics2");
    let tmp = physics1;
    let physics1 = physics2.clone();
    let physics2 = tmp.clone();
    assert_eq!(physics1.name, "physics2");
    assert_eq!(physics2.name, "physics1");
}
