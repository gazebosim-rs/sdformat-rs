// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#![cfg(all(target_os = "linux", target_env = "gnu"))]
use sdformat::{ErrorCode, NoResources, load_str};

#[test]
fn raw_float_conversion_keeps_errno_and_thread_independence() {
    let threads: Vec<_>=(0..8).map(|_|std::thread::spawn(|| {
        for _ in 0..20 {
            errno::set_errno(errno::Errno(123));
            let sdf=load_str("<sdf version='1.12'><model name='m'><link name='l'><visual name='v'><material><ambient>0.1 0.2 0.3</ambient><render_order>0x1p2tail</render_order></material></visual></link></model></sdf>","threads",&NoResources).unwrap();
            let mat=sdf.models[0].links[0].visuals[0].material.as_deref().unwrap();
            assert_eq!(mat.ambient,[0.1_f32,0.2,0.3,1.]);assert_eq!(mat.render_order,4.0);
            assert_eq!(errno::errno(),errno::Errno(123));
        }
    })).collect();
    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn raw_float_second_locale_allocation_failure() {
    const CHILD: &str = "SDF_MATERIAL_LOCALE_FAILURE_CHILD";
    if std::env::var_os(CHILD).is_some() {
        errno::set_errno(errno::Errno(42));
        let error=load_str("<sdf version='1.12'><model name='m'><link name='l'><visual name='v'><material><ambient>0.1 0.2 0.3 1</ambient></material></visual></link></model></sdf>","fault",&NoResources).unwrap_err();
        assert_eq!(error.code, ErrorCode::Resource);
        assert_eq!(errno::errno(), errno::Errno(42));
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let library = directory.path().join("fail-second-locale.so");
    let compile = std::process::Command::new("cc")
        .args(["-shared", "-fPIC", "-Wall", "-Wextra", "-Werror"])
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/visual_locale_failure.c"
        ))
        .args(["-ldl", "-o"])
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
            "raw_float_second_locale_allocation_failure",
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
