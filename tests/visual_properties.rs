// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#![cfg(all(target_os = "linux", target_env = "gnu"))]
use sdformat::*;
#[test]
fn native_visual_properties_and_float_getter_bits() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("visual_properties.json")).unwrap();
    let mut nonfinite_gaps = 0;
    for case in cases.as_array().unwrap() {
        let result = load_str(case["xml"].as_str().unwrap(), "visual.sdf", &NoResources);
        if ["transparency_inf", "transparency_nan"].contains(&case["id"].as_str().unwrap()) {
            assert!(case["accepted"].as_bool().unwrap());
            assert_eq!(result.unwrap_err().code, ErrorCode::InvalidValue);
            nonfinite_gaps += 1;
            continue;
        }
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{case}"
        );
        if let Ok(sdf) = result {
            let properties = &sdf.models[0].links[0].visuals[0].properties;
            if case["id"] == "absent" {
                assert_eq!(properties, &VisualProperties::default());
            }
            assert_eq!(
                properties.cast_shadows,
                case["cast_shadows"].as_bool().unwrap()
            );
            assert_eq!(
                properties.transparency.to_bits() as u64,
                case["transparency_bits"].as_u64().unwrap(),
                "{case}"
            );
            assert_eq!(
                properties.visibility_flags as u64,
                case["visibility_flags"].as_u64().unwrap()
            );
            assert_eq!(
                properties.laser_retro.is_some(),
                case["has_laser_retro"].as_bool().unwrap()
            );
            assert_eq!(
                properties.laser_retro.unwrap_or(0.).to_bits(),
                case["laser_retro_bits"].as_u64().unwrap()
            );
        }
    }
    assert_eq!(nonfinite_gaps, 2);
}

#[test]
fn visual_float_conversion_preserves_errno_concurrently() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("visual_properties.json")).unwrap();
    std::thread::scope(|scope| {
        for id in 0..8 {
            let cases = &cases;
            scope.spawn(move || {
                for _ in 0..8 {
                    for case in cases.as_array().unwrap() {
                        errno::set_errno(errno::Errno(100 + id));
                        let _ = load_str(
                            case["xml"].as_str().unwrap(),
                            "concurrent-visual",
                            &NoResources,
                        );
                        assert_eq!(errno::errno(), errno::Errno(100 + id));
                    }
                }
            });
        }
    });
}

#[test]
fn visual_second_locale_allocation_failure() {
    const CHILD: &str = "SDF_VISUAL_SECOND_LOCALE_FAILURE";
    if std::env::var_os(CHILD).is_some() {
        errno::set_errno(errno::Errno(123));
        let xml = "<sdf version='1.12'><model name='m'><link name='l'><visual name='v'><transparency>0.5</transparency></visual></link></model></sdf>";
        assert_eq!(
            load_str(xml, "resource", &NoResources).unwrap_err().code,
            ErrorCode::Resource
        );
        assert_eq!(errno::errno(), errno::Errno(123));
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let temporary = tempfile::tempdir().unwrap();
    let library = temporary.path().join("visual-locale-failure.so");
    let compile = std::process::Command::new("cc")
        .args(["-shared", "-fPIC", "-Wall", "-Wextra", "-Werror"])
        .arg(root.join("tests/visual_locale_failure.c"))
        .arg("-ldl")
        .arg("-o")
        .arg(&library)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "visual_second_locale_allocation_failure",
            "--test-threads=1",
        ])
        .env(CHILD, "1")
        .env("LD_PRELOAD", library)
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{:?} {} {}",
        child.status,
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
    let output = String::from_utf8(child.stdout).unwrap();
    assert!(output.contains("running 1 test") && output.contains("1 passed; 0 failed"));
}

#[test]
fn stored_double_midpoint_uses_native_text_rounding() {
    let xml = "<sdf version='1.12'><model name='m'><link name='l'><visual name='v'><transparency>0.5000000298023223876953125</transparency></visual></link></model></sdf>";
    let sdf = load_str(xml, "midpoint", &NoResources).unwrap();
    assert_eq!(
        sdf.models[0].links[0].visuals[0]
            .properties
            .transparency
            .to_bits(),
        0x3f000001
    );
    let midpoint = f64::from_bits(0x3fe0000010000000);
    assert_eq!((midpoint as f32).to_bits(), 0x3f000000);
}
