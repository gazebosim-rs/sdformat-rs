/*
 * Copyright (C) 2022 Open Source Robotics Foundation
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
// Fixed height observations from five pinned DOMPolyline ownership originals.
use sdformat::{Polyline, Source};

fn polyline(height: f64) -> Polyline {
    Polyline {
        height,
        points: Vec::new(),
        source: Source {
            uri: "owned-polyline".into(),
            line: 1,
            column: 1,
        },
    }
}

fn assert_height(actual: f64, expected: f64) {
    // These original heights are positive finite values; EXPECT_DOUBLE_EQ allows 4 ULP.
    assert!(actual.is_finite() && actual.is_sign_positive());
    assert!(actual.to_bits().abs_diff(expected.to_bits()) <= 4);
    assert_eq!(actual.to_bits(), expected.to_bits()); // Supplemental exact retention.
}

#[test]
fn original_dom_polyline_move_constructor() {
    let original = polyline(0.2);
    let moved = original;
    assert_height(moved.height, 0.2);
}

#[test]
fn original_dom_polyline_copy_constructor() {
    let original = polyline(0.2);
    let copied = original.clone();
    assert_height(copied.height, 0.2);
}

#[test]
fn original_dom_polyline_copy_assignment_operator() {
    let original = polyline(0.2);
    let mut copied = polyline(1.0);
    copied.clone_from(&original);
    assert_height(copied.height, 0.2);
}

#[test]
fn original_dom_polyline_move_assignment_operator() {
    let original = polyline(0.2);
    let mut moved = polyline(1.0);
    let previous = std::mem::replace(&mut moved, original);
    assert_height(previous.height, 1.0); // Supplemental replacement control.
    assert_height(moved.height, 0.2);
}

#[test]
fn original_dom_polyline_copy_assignment_after_move() {
    let first = polyline(0.1);
    let second = polyline(0.2);
    let tmp = first;
    let first = second.clone();
    let second = tmp.clone();
    assert_height(first.height, 0.2);
    assert_height(second.height, 0.1);
}
