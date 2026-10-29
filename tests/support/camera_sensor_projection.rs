// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use crate::camera_projection;
use camera_projection::hex;
use sdformat::*;
pub fn projection(value: &Sensor) -> Vec<String> {
    let mut output = vec![
        "OK".into(),
        hex(&value.name),
        hex(&value.kind),
        hex(&value.frame_id),
        hex(&value.topic),
        hex(&value.pose.relative_to),
        value.update_rate.to_bits().to_string(),
        u8::from(value.enable_metrics).to_string(),
        u8::from(value.always_on).to_string(),
        u8::from(value.visualize).to_string(),
    ];
    output.extend(
        value
            .pose
            .transform
            .translation
            .into_iter()
            .chain(value.pose.transform.rotation)
            .map(|n| n.to_bits().to_string()),
    );
    output.extend(
        camera_projection::projection(value.camera().unwrap())
            .into_iter()
            .skip(1),
    );
    output
}
pub fn compare(actual: &[String], expected: &[String]) {
    assert_eq!(actual.len(), 84);
    assert_eq!(expected.len(), 84);
    for (i, (a, b)) in actual[..17].iter().zip(&expected[..17]).enumerate() {
        if (10..17).contains(&i) {
            let a = f64::from_bits(a.parse().unwrap());
            let b = f64::from_bits(b.parse().unwrap());
            assert!((a - b).abs() <= 1e-3, "sensor pose {i}: {a} != {b}");
        } else {
            assert_eq!(a, b, "sensor token {i}");
        }
    }
    let mut camera_actual = vec!["OK".into()];
    camera_actual.extend_from_slice(&actual[17..]);
    let mut camera_expected = vec!["OK".into()];
    camera_expected.extend_from_slice(&expected[17..]);
    camera_projection::compare(&camera_actual, &camera_expected);
}
