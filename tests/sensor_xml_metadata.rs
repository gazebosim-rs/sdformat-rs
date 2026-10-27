// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Qualified by the pinned native sensor Element capture, not runtime behavior.
use sdformat::*;

fn sensor(body: &str) -> Result<Sensor, Diagnostic> {
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='imu'>{body}</sensor></link></model></sdf>"
    );
    Ok(load_str(&xml, "sensor-metadata.sdf", &NoResources)?
        .models
        .remove(0)
        .links
        .remove(0)
        .sensors
        .remove(0))
}

#[test]
fn native_canonical_booleans_and_unknown_legacy_xml_remain_distinct() {
    let omitted = sensor("").unwrap();
    assert!(!omitted.always_on);
    assert!(!omitted.visualize);
    assert!(omitted.extensions.is_empty());
    assert!(!Sensor::default().visualize);
    assert!(Sensor::default().extensions.is_empty());
    let raw = "<alwaysOn a='x&amp;y'>not-a-bool<child/>tail</alwaysOn>";
    let mut value = sensor(&format!(
        "<always_on>false</always_on>{raw}<alwaysOn>1</alwaysOn><visualize>true</visualize>"
    ))
    .unwrap();
    assert!(
        !value.always_on,
        "unknown spelling must not alias canonical flag"
    );
    assert!(value.visualize);
    assert_eq!(value.extensions.len(), 2);
    assert_eq!(value.extensions[0].tag, "alwaysOn");
    assert_eq!(value.extensions[0].attribute("a"), Some("x&y"));
    assert_eq!(value.extensions[0].xml, raw);
    assert_eq!(value.extensions[0].source.uri, "sensor-metadata.sdf");
    assert_eq!(value.extensions[0].source.line, 1);
    assert!(value.extensions[0].source.column > 0);
    assert_eq!(value.extensions[1].xml, "<alwaysOn>1</alwaysOn>");
    let copy = value.clone();
    value.extensions[0].xml.clear();
    drop(value);
    assert_eq!(copy.extensions[0].xml, raw);
    assert!(copy.visualize);
    assert!(!copy.always_on);
    assert!(sensor("<always_on>true</always_on>").unwrap().always_on);
}

#[test]
fn known_boolean_validation_and_unqualified_sensor_boundaries_are_preserved() {
    assert_eq!(
        sensor("<visualize>not-a-bool</visualize>")
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
    assert_eq!(
        sensor("<always_on>not-a-bool</always_on>")
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
    assert!(sensor("<alwaysOn>not-a-bool</alwaysOn>").is_ok());
    assert_eq!(
        sensor("<other_unknown>1</other_unknown>").unwrap_err().code,
        ErrorCode::Unsupported
    );
    // Native first-value repeated known scalars remain an explicit loader subset gap.
    assert!(sensor("<visualize>true</visualize><visualize>false</visualize>").is_err());
    assert!(sensor("<always_on>true</always_on><always_on>false</always_on>").is_err());
}
