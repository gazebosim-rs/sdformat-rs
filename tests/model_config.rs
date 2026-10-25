// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, ModelConfigWarningKind, select_model_config};
use serde_json::Value;

#[test]
fn native_config_selection_corpus() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("model_config.json")).unwrap();
    for case in cases {
        let id = case["id"].as_str().unwrap();
        let result = select_model_config(case["xml"].as_str().unwrap(), "model.config");
        let stdout = case["stdout"].as_str().unwrap();
        let tokens: Vec<_> = stdout.split_whitespace().collect();
        if matches!(id, "missing_name_future" | "empty_name") {
            // Native warning construction dereferences missing/null name text.
            // The new pure Result API safely retains selection/warnings instead.
            let selection = result.unwrap_or_else(|error| panic!("{id}: {error}"));
            assert_eq!(selection.file_name, "file.sdf", "{id}");
            let (version, warning) = if id == "missing_name_future" {
                (
                    Some("2.0"),
                    ModelConfigWarningKind::NewerVersion("2.0".into()),
                )
            } else {
                (None, ModelConfigWarningKind::MissingVersion)
            };
            assert_eq!(selection.declared_version.as_deref(), version, "{id}");
            assert_eq!(selection.warnings.len(), 1, "{id}");
            assert_eq!(selection.warnings[0].kind, warning, "{id}");
        } else {
            assert_eq!(
                case["returncode"].as_i64(),
                Some(0),
                "unexpected native failure: {id}"
            );
            if let Some(gap) = case.get("gap") {
                assert_eq!(tokens[0], "ok", "{id}");
                assert_eq!(tokens[2], "0", "{id}: native must accept the gap control");
                let error = result.unwrap_err();
                assert_eq!(
                    format!("{:?}", error.code),
                    gap["rust_error"].as_str().unwrap(),
                    "{id}"
                );
            } else if tokens[0] == "exception" {
                assert_eq!(result.unwrap_err().code, ErrorCode::InvalidValue, "{id}");
            } else if tokens[3..].contains(&"40") {
                let error = result.unwrap_err();
                assert!(
                    matches!(error.code, ErrorCode::Missing | ErrorCode::Xml),
                    "{id}: {error:?}"
                );
            } else {
                let selected = result.unwrap_or_else(|error| panic!("{id}: {error}"));
                let name = if tokens[1] == "-" {
                    String::new()
                } else {
                    let bytes: Vec<_> = tokens[1]
                        .as_bytes()
                        .chunks_exact(2)
                        .map(|pair| {
                            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()
                        })
                        .collect();
                    String::from_utf8(bytes).unwrap()
                };
                assert_eq!(selected.file_name, name, "{id}");
                assert_eq!(
                    selected.warnings.len(),
                    tokens[2].parse::<usize>().unwrap(),
                    "{id}"
                );
            }
        }
    }
}

#[test]
fn warnings_and_selected_attribute_preserve_order_and_source() {
    let selection = select_model_config("<model>\n<sdf>first.sdf</sdf>\n<sdf version='2.0'>future.sdf</sdf>\n<sdf version='3'>other.sdf</sdf>\n</model>", "cached/model.config").unwrap();
    assert_eq!(selection.file_name, "first.sdf");
    assert_eq!(selection.declared_version, None);
    assert_eq!(selection.warnings.len(), 3);
    for (warning, (line, kind)) in selection.warnings.iter().zip([
        (3, ModelConfigWarningKind::NewerVersion("2.0".into())),
        (4, ModelConfigWarningKind::NewerVersion("3".into())),
        (2, ModelConfigWarningKind::MissingVersion),
    ]) {
        assert_eq!(warning.kind, kind);
        assert_eq!(warning.source.uri, "cached/model.config");
        assert_eq!(warning.source.line, line);
        assert_eq!(warning.source.column, 1);
    }
    let versioned = select_model_config(
        "<model><sdf version='1.12.0-a'>a.sdf</sdf><sdf version='1.12'>b.sdf</sdf></model>",
        "m",
    )
    .unwrap();
    assert_eq!(versioned.file_name, "b.sdf");
    assert_eq!(versioned.declared_version.as_deref(), Some("1.12"));
    assert!(versioned.warnings.is_empty());
}
