// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, Frame, Light, load_frame_str, load_light_str};

fn light() -> Light {
    Light {
        name: "numeric".into(),
        ..Light::default()
    }
}

#[test]
fn light_scalar_subnormals_reject_before_emission() {
    let setters: [fn(&mut Light, f64); 8] = [
        |l, v| l.intensity = v,
        |l, v| l.attenuation_range = v,
        |l, v| l.linear_attenuation_factor = v,
        |l, v| l.constant_attenuation_factor = v,
        |l, v| l.quadratic_attenuation_factor = v,
        |l, v| l.spot_inner_angle = v,
        |l, v| l.spot_outer_angle = v,
        |l, v| l.spot_falloff = v,
    ];
    for set in setters {
        let mut value = light();
        set(&mut value, 1e-310);
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
}

#[test]
fn light_color_subnormals_reject_before_emission() {
    for channel in 0..4 {
        let mut value = light();
        value.diffuse[channel] = 1e-40;
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        let mut value = light();
        value.specular[channel] = 1e-40;
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
}

#[test]
fn frame_and_light_pose_subnormals_reject_before_emission() {
    for index in 0..7 {
        let mut frame = Frame {
            name: "numeric".into(),
            ..Frame::default()
        };
        if index < 3 {
            frame.pose.transform.translation[index] = 1e-310;
        } else {
            let component = index - 3;
            frame.pose.transform.rotation = [0.; 4];
            frame.pose.transform.rotation[(component + 1) % 4] = 1.;
            frame.pose.transform.rotation[component] = 1e-310;
        }
        let mut value = light();
        value.pose = frame.pose.clone();
        assert_eq!(frame.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
}

#[test]
fn zero_and_smallest_normal_scalars_still_roundtrip() {
    for scalar in [0., -0., f64::MIN_POSITIVE] {
        let mut value = light();
        value.intensity = scalar;
        let loaded = load_light_str(&value.to_xml().unwrap(), "light").unwrap();
        assert_eq!(loaded.intensity.to_bits(), scalar.to_bits());
        let mut frame = Frame {
            name: "numeric".into(),
            ..Frame::default()
        };
        frame.pose.transform.translation[0] = scalar;
        let loaded = load_frame_str(&frame.to_xml().unwrap(), "frame").unwrap();
        assert_eq!(
            loaded.pose.transform.translation[0].to_bits(),
            scalar.to_bits()
        );
    }
    let mut value = light();
    value.diffuse[0] = f32::MIN_POSITIVE;
    let loaded = load_light_str(&value.to_xml().unwrap(), "light").unwrap();
    assert_eq!(loaded.diffuse[0].to_bits(), f32::MIN_POSITIVE.to_bits());
}

#[test]
fn direction_keeps_its_distinct_vector_conversion() {
    let mut value = light();
    value.direction[0] = 1e-310;
    let loaded = load_light_str(&value.to_xml().unwrap(), "light").unwrap();
    assert_eq!(loaded.direction[0].to_bits(), value.direction[0].to_bits());
}
