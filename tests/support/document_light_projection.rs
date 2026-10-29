// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{Light, LightType, Transform};
pub fn hex(s: &str) -> String {
    if s.is_empty() {
        return "-".into();
    }
    s.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
pub fn projection(v: &Light) -> Vec<String> {
    let mut out = vec![
        "OK".into(),
        hex(&v.name),
        match v.light_type {
            LightType::Point => "1",
            LightType::Spot => "2",
            LightType::Directional => "3",
        }
        .into(),
    ];
    out.extend(
        v.pose
            .transform
            .translation
            .into_iter()
            .chain(v.pose.transform.rotation)
            .map(|n| n.to_bits().to_string()),
    );
    out.push(hex(&v.pose.relative_to));
    out.extend([v.light_on, v.visualize, v.cast_shadows].map(|v| u8::from(v).to_string()));
    out.extend(
        v.diffuse
            .into_iter()
            .chain(v.specular)
            .map(|n| n.to_bits().to_string()),
    );
    out.extend(
        [
            v.attenuation_range,
            v.spot_falloff,
            v.spot_inner_angle,
            v.spot_outer_angle,
            v.linear_attenuation_factor,
            v.constant_attenuation_factor,
            v.quadratic_attenuation_factor,
            v.direction[0],
            v.direction[1],
            v.direction[2],
            v.intensity,
        ]
        .map(|n| n.to_bits().to_string()),
    );
    out
}
pub fn compare(actual: &[String], expected: &[String]) {
    assert_eq!(expected.len(), 33);
    assert_eq!(actual.len(), expected.len());
    for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
        if (3..10).contains(&i) {
            let a = f64::from_bits(a.parse().unwrap());
            let b = f64::from_bits(b.parse().unwrap());
            assert!((a - b).abs() <= 1e-3, "pose {i}: {a} != {b}");
        } else {
            assert_eq!(a, b, "field {i}");
        }
    }
}
pub fn pose(actual: Transform, expected: &[String]) {
    assert_eq!(expected.len(), 7);
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected)
    {
        let b = f64::from_bits(b.parse().unwrap());
        assert!((a - b).abs() <= 1e-3, "pose {a} != {b}");
    }
}
