// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use std::process::Command;

pub fn assert_quiet(test: &str, marker: &str, body: impl FnOnce()) {
    if std::env::var_os(marker).is_some() {
        body();
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture", "--color", "never"])
        .env(marker, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let prefix = format!(
        "\nrunning 1 test\ntest {test} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; "
    );
    let summary = stdout
        .strip_prefix(&prefix)
        .and_then(|text| text.strip_suffix("s\n\n"))
        .expect(&stdout);
    let (filtered, elapsed) = summary
        .split_once(" filtered out; finished in ")
        .expect(&stdout);
    filtered.parse::<usize>().unwrap();
    let elapsed = elapsed.parse::<f64>().unwrap();
    assert!(elapsed.is_finite() && elapsed >= 0.);
}
