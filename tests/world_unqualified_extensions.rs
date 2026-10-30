// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Direct unknown world metadata; known schema names remain explicit errors.
use sdformat::{ErrorCode, Resource, ResourceResolver, load_str};
use serde_json::Value;

struct Resolver;
impl ResourceResolver for Resolver {
    fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
        Ok(Resource {
            canonical_uri: "import.sdf".into(),
            xml: "<sdf version='1.12'><model name='m'><model name='n'><link name='l'/></model><v:imported/></model></sdf>".into(),
        })
    }
}

#[test]
fn unqualified_world_metadata_preserves_native_order_and_owned_source() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("world_unqualified_extensions.json")).unwrap();
    assert_eq!(rows.len(), 21);
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        if !(id.starts_with("duplicates_")
            || id.starts_with("order_")
            || id.starts_with("multiline_")
            || id.starts_with("old_unknown_"))
        {
            assert_eq!(
                load_str(xml, "root.sdf", &Resolver).unwrap_err().code,
                ErrorCode::Unsupported,
                "{id}"
            );
            continue;
        }
        let sdf = load_str(xml, "root.sdf", &Resolver).unwrap();
        let world = &sdf.worlds[0];
        let expected: &[(&str, &str, &str)] = if id.starts_with("duplicates_") {
            &[
                (
                    "custom",
                    "<custom name='a' a='α&amp;β'>π<child/>tail</custom>",
                    "root.sdf",
                ),
                ("custom", "<custom name='b'/>", "root.sdf"),
                ("v:qualified", "<v:qualified/>", "root.sdf"),
                ("other", "<other/>", "root.sdf"),
            ]
        } else if id.starts_with("multiline_") {
            &[
                (
                    "vendor_meta",
                    "<vendor_meta note='α&amp;β'>\n  π<child/>tail\n </vendor_meta>",
                    "root.sdf",
                ),
                ("other", "<other/>", "root.sdf"),
            ]
        } else if id.starts_with("old_unknown_") {
            &[("unknown", "<unknown/>", "root.sdf")]
        } else {
            &[
                ("v:imported", "<v:imported/>", "import.sdf"),
                ("before", "<before/>", "root.sdf"),
                ("v:direct", "<v:direct/>", "root.sdf"),
                ("after", "<after/>", "root.sdf"),
                ("before", "<before/>", "root.sdf"),
            ]
        };
        assert_eq!(world.extensions.len(), expected.len(), "{id}");
        let native = row["stdout"].as_str().unwrap();
        let mut native_at = 0;
        let mut input_at = 0;
        for (actual, (tag, raw, uri)) in world.extensions.iter().zip(expected) {
            assert_eq!(
                (&*actual.tag, &*actual.xml, &*actual.source.uri),
                (*tag, *raw, *uri),
                "{id}"
            );
            if *uri == "root.sdf" {
                input_at += xml[input_at..].find(raw).unwrap();
                let prefix = &xml[..input_at];
                assert_eq!(
                    actual.source.line as usize,
                    prefix.bytes().filter(|b| *b == b'\n').count() + 1
                );
                assert_eq!(
                    actual.source.column as usize,
                    prefix.rsplit('\n').next().unwrap().chars().count() + 1
                );
                input_at += raw.len();
            }
            let marker = format!("<{tag}");
            native_at += native[native_at..]
                .find(&marker)
                .unwrap_or_else(|| panic!("{id}: {tag}"))
                + marker.len();
        }
        if id.starts_with("duplicates_") {
            assert_eq!(world.extensions[0].attribute("a"), Some("α&β"));
            assert_eq!(world.extensions[0].attribute("name"), Some("a"));
        }
        let expected_warnings = if id.starts_with("old_unknown_") {
            1
        } else if id.starts_with("multiline_") {
            2
        } else {
            3
        };
        assert_eq!(
            row["stderr"].as_str().unwrap().matches("Warning").count(),
            expected_warnings
        );
        if id.starts_with("multiline_") {
            assert_eq!(world.extensions[0].attribute("note"), Some("α&β"));
        }
        let mut snapshot = world.extensions.clone();
        snapshot[0].xml.clear();
        assert!(!world.extensions[0].xml.is_empty());
    }
}

#[test]
fn known_schema_spellings_are_not_admitted_as_world_metadata() {
    // Genuine supported world children are validated by their existing paths.
    let admitted = [
        "model",
        "light",
        "include",
        "frame",
        "joint",
        "gravity",
        "wind",
        "magnetic_field",
        "plugin",
        "physics",
        "atmosphere",
    ];
    let names = include_str!("../data/world_known_schema_names.txt");
    for name in names.lines().filter(|name| !name.starts_with('#')) {
        if admitted.contains(&name) {
            continue;
        }
        let xml = format!("<sdf version='1.12'><world name='w'><{name}/></world></sdf>");
        assert_eq!(
            load_str(&xml, "root.sdf", &Resolver).unwrap_err().code,
            ErrorCode::Unsupported,
            "{name}"
        );
    }
    for name in ["Link", "SCENE", "dT"] {
        let xml = format!("<sdf version='1.12'><world name='w'><{name}/></world></sdf>");
        assert_eq!(
            load_str(&xml, "root.sdf", &Resolver).unwrap_err().code,
            ErrorCode::Unsupported,
            "{name}"
        );
    }
}

#[test]
fn exact_prior_unqualified_direct_inputs_retain_all_children() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("world_direct_extensions.json")).unwrap();
    let mut checked = 0;
    for row in rows
        .iter()
        .filter(|row| row["id"].as_str().unwrap().starts_with("unknown_"))
    {
        let id = row["id"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        let sdf = load_str(xml, "root.sdf", &Resolver).unwrap();
        let entries: Vec<_> = sdf.worlds[0]
            .extensions
            .iter()
            .map(|value| (&*value.tag, &*value.xml, &*value.source.uri))
            .collect();
        assert_eq!(
            entries,
            [
                ("plain", "<plain>keep?</plain>", "root.sdf"),
                ("v:ok", "<v:ok/>", "root.sdf"),
                ("plain", "<plain>twice</plain>", "root.sdf"),
            ],
            "{id}"
        );
        let native = row["stdout"].as_str().unwrap();
        let mut position = 0;
        for marker in ["<plain>keep?</plain>", "<v:ok/>", "<plain>twice</plain>"] {
            position += native[position..].find(marker).unwrap() + marker.len();
        }
        assert_eq!(
            row["stderr"].as_str().unwrap().matches("Warning").count(),
            2
        );
        checked += 1;
    }
    assert_eq!(checked, 3);
}
