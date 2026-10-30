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
use sdformat::*;
#[path = "support/quiet.rs"]
mod quiet;
fn check() {
    let mut polyline = load_polyline_str("<polyline/>", "configured.xml").unwrap();
    polyline.points.clear();
    polyline.height = 1.2;
    for point in [[1., 2.], [3., 4.]] {
        let before = polyline.points.len();
        polyline.points.push(point);
        assert_eq!(polyline.points.len(), before + 1);
        assert_eq!(polyline.points.last(), Some(&point));
    }
    let xml = polyline.to_xml().unwrap();
    assert!(!xml.is_empty());
    let loaded = load_polyline_str(&xml, "polyline-original.xml").unwrap();
    assert!(polyline.height.to_bits().abs_diff(loaded.height.to_bits()) <= 4);
    assert_eq!(polyline.points.len(), loaded.points.len());
    assert_eq!(loaded.points.len(), 2);
    for index in 0..2 {
        for (a, b) in polyline.points[index].into_iter().zip(loaded.points[index]) {
            assert!((a - b).abs() <= 1e-3);
        }
    }
}
#[test]
fn original_polyline_to_element() {
    check();
}
#[test]
fn original_polyline_to_element_error_output() {
    quiet::assert_quiet(
        "original_polyline_to_element_error_output",
        "SDF_POLYLINE_QUIET_CHILD",
        check,
    );
}
