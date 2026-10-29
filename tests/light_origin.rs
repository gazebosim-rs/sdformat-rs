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
fn standalone_input_origin_survives_edits_and_output() {
    let mut value = load_light_str("\n  <light name='origin_light'/>", "caller-light.sdf").unwrap();
    let original = Some(source("caller-light.sdf", 2, 3));
    assert_eq!(value.source, original);
    value.cast_shadows = true;
    let cloned = value.clone();
    let output = value.to_xml().unwrap();
    assert_eq!(value, cloned);
    assert_eq!(value.source, original);
    let reloaded = load_light_str(&output, "emitted-light.sdf").unwrap();
    assert_eq!(reloaded.source, Some(source("emitted-light.sdf", 1, 1)));
    assert!(reloaded.cast_shadows);
}
#[test]
fn document_lights_require_explicit_type() {
    for (xml, column) in [
        (
            "<sdf version='1.12'><world name='w'><light name='l'/></world></sdf>",
            37,
        ),
        (
            "<sdf version='1.12'><model name='m'><link name='l'><light name='light'/></link></model></sdf>",
            52,
        ),
    ] {
        let error = load_str(xml, "document-light.sdf", &NoResources).unwrap_err();
        assert_eq!(error.code, ErrorCode::Missing);
        assert_eq!(error.source.uri, "document-light.sdf");
        assert_eq!(
            error.source,
            Source {
                uri: "document-light.sdf".into(),
                line: 1,
                column
            }
        );
    }
}
#[test]
fn constructed_and_loaded_presence_has_literal_input_origin() {
    let constructed = Light {
        name: "origin_light".into(),
        ..Light::default()
    };
    assert!(constructed.source.is_none());
    let xml = constructed.to_xml().unwrap();
    assert!(!xml.is_empty());
    assert!(constructed.source.is_none());
    let loaded = load_light_str(&xml, "light-roundtrip").unwrap();
    let literal = Some(source("light-roundtrip", 1, 1));
    assert_eq!(loaded.source, literal);
    let mut expected = constructed.clone();
    expected.source = literal;
    assert_eq!(loaded, expected);
}
