// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
pub fn hex(value: &str) -> String {
    if value.is_empty() {
        return "-".into();
    }
    value
        .as_bytes()
        .iter()
        .map(|n| format!("{n:02x}"))
        .collect()
}
fn number(values: &mut Vec<String>, value: f64) {
    values.push(value.to_bits().to_string());
}
pub fn projection(c: &Camera) -> Vec<String> {
    let format = match c.pixel_format {
        CameraPixelFormat::RgbInt8 => "RGB_INT8",
        CameraPixelFormat::LInt8 => "L_INT8",
    };
    let mut v = vec![
        "OK".into(),
        hex(&c.name),
        hex(&c.pose.relative_to),
        hex(&c.save_frames_path),
        hex(&c.camera_info_topic),
        hex(&c.trigger_topic),
        hex(format),
    ];
    v.extend([
        u8::from(c.triggered).to_string(),
        u8::from(c.save_frames).to_string(),
        c.image_width.to_string(),
        c.image_height.to_string(),
        c.anti_aliasing.to_string(),
        c.visibility_mask.to_string(),
    ]);
    for n in [c.horizontal_fov, c.near_clip, c.far_clip]
        .into_iter()
        .chain(c.pose.transform.translation)
        .chain(c.pose.transform.rotation)
    {
        number(&mut v, n);
    }
    v.extend([
        u8::from(c.has_depth_near_clip).to_string(),
        u8::from(c.has_depth_far_clip).to_string(),
    ]);
    number(&mut v, c.depth_near_clip);
    number(&mut v, c.depth_far_clip);
    v.extend([
        u8::from(c.has_segmentation_type).to_string(),
        u8::from(c.has_bounding_box_type).to_string(),
        hex(&c.segmentation_type),
        hex(&c.bounding_box_type),
    ]);
    v.push(
        match c.image_noise.kind {
            NoiseKind::None => "0",
            NoiseKind::Gaussian => "1",
            NoiseKind::GaussianQuantized => "2",
        }
        .into(),
    );
    for n in [
        c.image_noise.mean,
        c.image_noise.stddev,
        c.image_noise.bias_mean,
        c.image_noise.bias_stddev,
        c.image_noise.dynamic_bias_stddev,
        c.image_noise.dynamic_bias_correlation_time,
        c.image_noise.precision,
        c.distortion.k1,
        c.distortion.k2,
        c.distortion.k3,
        c.distortion.p1,
        c.distortion.p2,
        c.distortion.center[0],
        c.distortion.center[1],
    ] {
        number(&mut v, n);
    }
    v.extend([
        hex(&c.lens.kind),
        hex(&c.lens.function),
        u8::from(c.lens.scale_to_hfov).to_string(),
        c.lens.environment_texture_size.to_string(),
        u8::from(c.lens.has_intrinsics).to_string(),
        u8::from(c.lens.has_projection).to_string(),
    ]);
    for n in [
        c.lens.c1,
        c.lens.c2,
        c.lens.c3,
        c.lens.focal_length,
        c.lens.cutoff_angle,
        c.lens.intrinsics.fx,
        c.lens.intrinsics.fy,
        c.lens.intrinsics.cx,
        c.lens.intrinsics.cy,
        c.lens.intrinsics.skew,
        c.lens.projection.fx,
        c.lens.projection.fy,
        c.lens.projection.cx,
        c.lens.projection.cy,
        c.lens.projection.tx,
        c.lens.projection.ty,
    ] {
        number(&mut v, n);
    }
    v
}
pub fn compare(actual: &[String], expected: &[String]) {
    assert_eq!(actual.len(), 68);
    assert_eq!(expected.len(), 68);
    for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
        if (16..23).contains(&i) {
            let a = f64::from_bits(a.parse().unwrap());
            let b = f64::from_bits(b.parse().unwrap());
            assert!((a - b).abs() <= 1e-3, "pose {i}: {a} != {b}");
        } else {
            assert_eq!(a, b, "token {i}");
        }
    }
}
