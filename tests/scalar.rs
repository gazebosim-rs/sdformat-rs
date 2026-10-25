// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#![cfg(all(target_os = "linux", target_env = "gnu"))]
use sdformat::*;

fn cases() -> serde_json::Value {
    serde_json::from_str(include_str!("scalar.json")).unwrap()
}

#[test]
fn scalar_native_range_prefix_and_bits() {
    let cases = cases();
    assert_eq!(cases.as_array().unwrap().len(), 63);
    let mut documented_nonfinite_gaps = 0;
    for case in cases.as_array().unwrap() {
        let result = load_str(case["xml"].as_str().unwrap(), "scalar.sdf", &NoResources);
        if !case["supported"].as_bool().unwrap() {
            assert!(case["native_accepted"].as_bool().unwrap());
            assert_eq!(result.unwrap_err().code, ErrorCode::InvalidValue);
            documented_nonfinite_gaps += 1;
            continue;
        }
        assert_eq!(
            result.is_ok(),
            case["native_accepted"].as_bool().unwrap(),
            "{case}"
        );
        if let Ok(sdf) = result {
            let value = match case["kind"].as_str().unwrap() {
                "pose" => sdf.models[0].pose.transform.translation[0],
                "vector" => {
                    let Geometry::Mesh(mesh) = &sdf.models[0].links[0].collisions[0].geometry
                    else {
                        panic!("mesh expected")
                    };
                    mesh.scale[0]
                }
                _ => {
                    sdf.models[0].links[0].sensors[0]
                        .imu()
                        .unwrap()
                        .angular_velocity[0]
                        .mean
                }
            };
            assert_eq!(value.to_bits(), case["bits"].as_u64().unwrap(), "{case}");
        }
    }
    assert_eq!(documented_nonfinite_gaps, 2);
}

#[test]
fn scalar_conversion_preserves_errno_concurrently() {
    let cases = cases();
    let cases = std::sync::Arc::new(cases);
    let workers: Vec<_> = (0..8)
        .map(|id| {
            let cases = cases.clone();
            std::thread::spawn(move || {
                for _ in 0..8 {
                    for case in cases.as_array().unwrap() {
                        let sentinel = errno::Errno(100 + id);
                        errno::set_errno(sentinel);
                        let result =
                            load_str(case["xml"].as_str().unwrap(), "errno.sdf", &NoResources);
                        assert_eq!(errno::errno(), sentinel);
                        assert_eq!(
                            result.is_ok(),
                            case["native_accepted"].as_bool().unwrap()
                                && case["supported"].as_bool().unwrap()
                        );
                    }
                }
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
}

// This test-only ABI snapshot does not change locale; production unsafe remains
// confined to src/number.rs. No test in this process calls setlocale with a name.
#[allow(unsafe_code)]
mod locale_snapshot {
    pub fn numeric() -> Vec<u8> {
        // SAFETY: A null locale queries current LC_NUMERIC. This test process has
        // no global locale writer; copy its valid NUL-terminated result at once.
        unsafe {
            let name = libc::setlocale(libc::LC_NUMERIC, std::ptr::null());
            assert!(!name.is_null());
            std::ffi::CStr::from_ptr(name).to_bytes().to_vec()
        }
    }
}

#[test]
fn scalar_loading_does_not_change_global_numeric_locale() {
    let before = locale_snapshot::numeric();
    for case in cases().as_array().unwrap() {
        let _ = load_str(case["xml"].as_str().unwrap(), "locale.sdf", &NoResources);
    }
    let visuals: serde_json::Value =
        serde_json::from_str(include_str!("visual_properties.json")).unwrap();
    for case in visuals.as_array().unwrap() {
        let _ = load_str(case["xml"].as_str().unwrap(), "locale-visual", &NoResources);
    }
    assert_eq!(locale_snapshot::numeric(), before);
}

#[test]
fn locale_allocation_failure() {
    const CHILD: &str = "SDF_SCALAR_LOCALE_FAILURE_CHILD";
    if std::env::var_os(CHILD).is_some() {
        errno::set_errno(errno::Errno(123));
        let case = &cases()[5];
        let result = load_str(case["xml"].as_str().unwrap(), "locale.sdf", &NoResources);
        assert_eq!(result.unwrap_err().code, ErrorCode::Resource);
        assert_eq!(errno::errno(), errno::Errno(123));
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let temporary = tempfile::tempdir().unwrap();
    let library = temporary.path().join("scalar-locale-failure.so");
    let compile = std::process::Command::new("cc")
        .args(["-shared", "-fPIC", "-Wall", "-Wextra", "-Werror"])
        .arg(root.join("tests/locale_failure.c"))
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
        .args(["--exact", "locale_allocation_failure", "--test-threads=1"])
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
    assert!(
        output.contains("running 1 test") && output.contains("1 passed; 0 failed"),
        "{output}"
    );
}
