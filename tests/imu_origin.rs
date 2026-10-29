// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
fn source(uri: &str, line: u32, column: u32) -> Source {
    Source {
        uri: uri.into(),
        line,
        column,
    }
}
#[test]
fn explicit_declaration_origin_survives_edits_and_immutable_output() {
    let mut value = load_imu_str("\n  <imu/>", "caller-imu.sdf").unwrap();
    let original = Some(source("caller-imu.sdf", 2, 3));
    assert_eq!(value.source, original);
    value.gravity_dir_x = [1., 2., 3.];
    let cloned = value.clone();
    let output = value.to_xml().unwrap();
    assert_eq!(value, cloned);
    assert_eq!(value.source, original);
    let reloaded = load_imu_str(&output, "emitted.sdf").unwrap();
    assert_eq!(reloaded.source, Some(source("emitted.sdf", 1, 1)));
    assert_eq!(reloaded.gravity_dir_x, [1., 2., 3.]);
    let constructed = Imu::default();
    assert!(constructed.source.is_none());
    let output = constructed.to_xml().unwrap();
    assert!(!output.is_empty());
    assert!(constructed.source.is_none());
}
#[test]
fn sensor_omitted_configuration_has_no_explicit_input_declaration() {
    let mut omitted = Sensor::default();
    omitted
        .load_element_str("<sensor name='s' type='imu'/>", "omitted.sdf", &NoResources)
        .unwrap();
    assert!(omitted.imu().unwrap().source.is_none());
    let mut explicit = Sensor::default();
    explicit
        .load_element_str(
            "\n<sensor name='s' type='imu'>\n  <imu/>\n</sensor>",
            "sensor.sdf",
            &NoResources,
        )
        .unwrap();
    assert_eq!(
        explicit.imu().unwrap().source,
        Some(source("sensor.sdf", 3, 3))
    );
}
#[test]
fn full_document_declarations_use_the_input_location() {
    let xml = "<sdf version='1.12'>\n<model name='m'><link name='l'><sensor name='s' type='imu'>\n  <imu/>\n</sensor></link></model></sdf>";
    let value = load_str(xml, "world.sdf", &NoResources).unwrap();
    assert_eq!(
        value.models[0].links[0].sensors[0].imu().unwrap().source,
        Some(source("world.sdf", 3, 3))
    );
}
