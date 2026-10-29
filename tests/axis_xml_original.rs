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
use std::process::Command;

fn original_body() {
    let mut axis = Axis {
        xyz: [0., 0., 1.],
        expressed_in: String::new(),
        lower: f64::NEG_INFINITY,
        upper: f64::INFINITY,
        effort: f64::INFINITY,
        velocity: f64::INFINITY,
        dynamics: AxisDynamics::default(),
        stiffness: 1e8,
        dissipation: 1.,
        mimic: None,
    };
    let errors = axis.try_set_xyz([0., 1., 0.]);
    assert!(errors.is_ok());
    axis.expressed_in = "test".into();
    assert!(errors.is_ok());
    axis.dynamics.damping = 0.2;
    axis.dynamics.friction = 1.3;
    axis.dynamics.spring_reference = 2.4;
    axis.dynamics.spring_stiffness = -1.2;
    axis.lower = -10.8;
    axis.upper = 123.4;
    axis.effort = 3.2;
    axis.velocity = 54.2;
    axis.stiffness = 1e2;
    axis.dissipation = 1.5;
    axis.mimic = Some(AxisMimic {
        joint: "test_joint".into(),
        axis: "axis2".into(),
        multiplier: 5.,
        offset: 1.,
        reference: 2.,
    });
    let emitted = axis.to_xml();
    assert!(emitted.is_ok());
    let xml = emitted.unwrap();
    if let Some(path) = std::env::var_os("SDF_AXIS_XML_EXPORT") {
        std::fs::write(path, &xml).unwrap();
    }
    let loaded = load_axis_str(&xml, "axis.xml");
    assert!(loaded.is_ok());
    let loaded = loaded.unwrap();
    assert!(xml.contains("<xyz "));
    assert!(xml.contains("<dynamics>"));
    assert!(xml.contains("<limit>"));
    for (actual, expected) in loaded.xyz.into_iter().zip([0., 1., 0.]) {
        assert!((actual - expected).abs() <= 1e-3);
    }
    assert_eq!(loaded.expressed_in, "test");
    assert!(loaded.dynamics.source.is_some());
    for (actual, expected) in [
        loaded.dynamics.damping,
        loaded.dynamics.friction,
        loaded.dynamics.spring_reference,
        loaded.dynamics.spring_stiffness,
        loaded.lower,
        loaded.upper,
        loaded.effort,
        loaded.velocity,
        loaded.stiffness,
        loaded.dissipation,
    ]
    .into_iter()
    .zip([0.2_f64, 1.3, 2.4, -1.2, -10.8, 123.4, 3.2, 54.2, 1e2, 1.5])
    {
        assert!(
            actual.to_bits().abs_diff(expected.to_bits()) <= 4,
            "{actual} versus {expected}"
        );
    }
    let mimic = loaded
        .mimic
        .as_ref()
        .expect("FindElement(mimic) is present");
    assert_eq!(mimic.joint, "test_joint");
    assert_eq!(mimic.axis, "axis2");
    for (actual, expected) in [mimic.multiplier, mimic.offset, mimic.reference]
        .into_iter()
        .zip([5_f64, 1., 2.])
    {
        assert!(actual.to_bits().abs_diff(expected.to_bits()) <= 4);
    }
}
#[test]
fn original_dom_joint_axis_to_element() {
    if std::env::var_os("SDF_AXIS_QUIET_CHILD").is_some() {
        original_body();
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "original_dom_joint_axis_to_element",
            "--nocapture",
            "--color",
            "never",
        ])
        .env("SDF_AXIS_QUIET_CHILD", "1")
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
    // Only libtest's fixed protocol and elapsed duration may reach stdout.
    let prefix = "\nrunning 1 test\ntest original_dom_joint_axis_to_element ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in ";
    let elapsed = stdout
        .strip_prefix(prefix)
        .and_then(|text| text.strip_suffix("s\n\n"))
        .expect(&stdout)
        .parse::<f64>()
        .unwrap();
    assert!(elapsed.is_finite() && elapsed >= 0.);
}
