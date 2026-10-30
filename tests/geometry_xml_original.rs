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
use sdformat::*;
#[path = "support/geometry_projection.rs"]
mod projection;
#[path = "support/quiet.rs"]
mod quiet;
fn check() {
    let values = [
        Geometry::Box { size: [1.; 3] },
        Geometry::Capsule {
            radius: 0.5,
            length: 1.,
        },
        Geometry::Cone {
            radius: 0.5,
            length: 1.,
        },
        Geometry::Cylinder {
            radius: 0.5,
            length: 1.,
        },
        Geometry::Ellipsoid { radii: [1.; 3] },
        Geometry::Sphere { radius: 1. },
        Geometry::Plane {
            normal: [0., 0., 1.],
            size: [1.; 2],
        },
    ];
    let fixtures = [
        include_str!("fixtures/geometry_box_native.xml"),
        include_str!("fixtures/geometry_capsule_native.xml"),
        include_str!("fixtures/geometry_cone_native.xml"),
        include_str!("fixtures/geometry_cylinder_native.xml"),
        include_str!("fixtures/geometry_ellipsoid_native.xml"),
        include_str!("fixtures/geometry_sphere_native.xml"),
        include_str!("fixtures/geometry_plane_native.xml"),
    ];
    for (value, fixture) in values.into_iter().zip(fixtures) {
        assert_eq!(
            load_geometry_str(fixture, "native-geometry.xml").unwrap(),
            value
        );
        let xml = value.to_xml().unwrap();
        assert!(!xml.is_empty());
        let again = load_geometry_str(&xml, "geometry-original.xml").unwrap();
        // The selected sum variant carries exactly one shape; every other
        // native shape pointer is absent and the polyline list is empty.
        assert_eq!(
            projection::projection(&again),
            projection::projection(&value)
        );
        assert_eq!(again, value);
    }
}
#[test]
fn original_geometry_to_element_supported_variant_subset() {
    check();
}
#[test]
fn original_geometry_to_element_error_output_supported_variant_subset() {
    quiet::assert_quiet(
        "original_geometry_to_element_error_output_supported_variant_subset",
        "SDF_GEOMETRY_QUIET_CHILD",
        check,
    );
}
