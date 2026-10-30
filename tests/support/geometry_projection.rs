// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::Geometry;
pub fn projection(value: &Geometry) -> Vec<String> {
    let (kind, mask, values) = match value {
        Geometry::Box { size } => (1, 1, size.to_vec()),
        Geometry::Sphere { radius } => (4, 32, vec![*radius]),
        Geometry::Cylinder { radius, length } => (2, 8, vec![*radius, *length]),
        Geometry::Capsule { radius, length } => (7, 2, vec![*radius, *length]),
        Geometry::Cone { radius, length } => (10, 4, vec![*radius, *length]),
        Geometry::Ellipsoid { radii } => (8, 16, radii.to_vec()),
        Geometry::Mesh(_) => (5, 128, vec![]),
        Geometry::Heightmap(_) => (6, 256, vec![]),
        Geometry::Plane { normal, size } => (3, 64, normal.iter().chain(size).copied().collect()),
        Geometry::Polyline(lines) => (9, if lines.is_empty() { 0 } else { 512 }, vec![]),
    };
    let mut tokens = vec!["VALUES".into(), kind.to_string(), mask.to_string()];
    tokens.extend(values.into_iter().map(|v| v.to_bits().to_string()));
    tokens
}
