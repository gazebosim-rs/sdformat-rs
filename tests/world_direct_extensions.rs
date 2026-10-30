// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, Resource, ResourceResolver, load_str};
use serde_json::Value;

struct Resolver;
impl ResourceResolver for Resolver {
    fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
        Ok(Resource {
            canonical_uri: "import.sdf".into(),
            xml: "<sdf version='1.12'><model name='m'><model name='n'><link name='l'/></model><v:dup k='M'/><v:imported/></model></sdf>".into(),
        })
    }
}

#[test]
fn direct_qualified_metadata_follows_imports_and_preserves_source() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("world_direct_extensions.json")).unwrap();
    assert_eq!(rows.len(), 13);
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        if id.starts_with("unknown_") {
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
            continue;
        }
        if id.starts_with("known_") {
            assert_eq!(
                load_str(xml, "root.sdf", &Resolver).unwrap_err().code,
                ErrorCode::InvalidValue
            );
            continue;
        }
        let sdf = load_str(xml, "root.sdf", &Resolver).unwrap();
        let world = &sdf.worlds[0];
        let expected: &[(&str, &str, &str)] = if id.starts_with("interleaved_") {
            &[
                ("v:dup", "<v:dup k='M'/>", "import.sdf"),
                ("v:imported", "<v:imported/>", "import.sdf"),
                ("v:dup", "<v:dup k='1'/>", "root.sdf"),
                ("v:dup", "<v:dup k='2'/>", "root.sdf"),
                (
                    "z:later",
                    "<z:later xmlns:z='urn:z' key='α&amp;β'>π<z:child/>tail</z:later>",
                    "root.sdf",
                ),
            ]
        } else if id.starts_with("multiline_") {
            &[
                (
                    "v:a",
                    "<v:a key='α&amp;β'>\n  text<v:child/>tail\n </v:a>",
                    "root.sdf",
                ),
                ("v:b", "<v:b/>", "root.sdf"),
                ("v:a", "<v:a key='last'/>", "root.sdf"),
            ]
        } else {
            &[
                ("x:item", "<x:item xmlns:x='urn:a' x:key='v'/>", "root.sdf"),
                ("x:item", "<x:item xmlns:x='urn:b' x:key='w'/>", "root.sdf"),
                ("unbound:item", "<unbound:item k='u'/>", "root.sdf"),
                ("xml:lang", "<xml:lang>en</xml:lang>", "root.sdf"),
            ]
        };
        assert_eq!(world.extensions.len(), expected.len(), "{id}");
        let native = row["stdout"].as_str().unwrap();
        let mut position = 0;
        for (actual, (tag, raw, uri)) in world.extensions.iter().zip(expected) {
            assert_eq!(
                (&*actual.tag, &*actual.xml, &*actual.source.uri),
                (*tag, *raw, *uri),
                "{id}"
            );
            if *uri == "root.sdf" {
                let prefix = &xml[..xml.find(raw).unwrap()];
                assert_eq!(
                    actual.source.line as usize,
                    prefix.bytes().filter(|b| *b == b'\n').count() + 1
                );
                assert_eq!(
                    actual.source.column as usize,
                    prefix.rsplit('\n').next().unwrap().chars().count() + 1
                );
            }
            let marker = format!("<{tag}");
            position += native[position..]
                .find(&marker)
                .unwrap_or_else(|| panic!("{id}: {tag}"))
                + marker.len();
        }
        if id.starts_with("interleaved_") {
            assert_eq!(world.extensions[4].attribute("key"), Some("α&β"));
        } else if id.starts_with("multiline_") {
            assert_eq!(world.extensions[0].attribute("key"), Some("α&β"));
            assert_eq!(world.extensions[2].attribute("key"), Some("last"));
        } else {
            assert_eq!(world.extensions[0].attribute("xmlns:x"), None);
            assert!(world.extensions[0].xml.contains("xmlns:x='urn:a'"));
            assert_eq!(world.extensions[1].attribute("x:key"), Some("w"));
        }
        let mut cloned = world.extensions.clone();
        cloned[0].xml.clear();
        assert!(!world.extensions[0].xml.is_empty());
    }
}

#[test]
fn exact_prior_direct_world_input_retains_imports_then_direct_metadata() {
    struct PriorResolver(String);
    impl ResourceResolver for PriorResolver {
        fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
            Ok(Resource {
                canonical_uri: "import-a.sdf".into(),
                xml: self.0.clone(),
            })
        }
    }
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("world_merge_extensions.json")).unwrap();
    let row = rows.iter().find(|row| row["id"] == "direct_world").unwrap();
    let xml = row["world"].as_str().unwrap();
    let sdf = load_str(
        xml,
        "root.sdf",
        &PriorResolver(row["resource_a"].as_str().unwrap().into()),
    )
    .unwrap();
    let world = &sdf.worlds[0];
    let entries: Vec<_> = world
        .extensions
        .iter()
        .map(|e| (&*e.tag, &*e.xml, &*e.source.uri))
        .collect();
    assert_eq!(
        entries,
        [
            ("v:alpha", "<v:alpha k='1'/>", "import-a.sdf"),
            ("v:alpha", "<v:alpha k='2'/>", "import-a.sdf"),
            ("v:beta", "<v:beta>text</v:beta>", "import-a.sdf"),
            ("v:parent", "<v:parent>own</v:parent>", "root.sdf"),
        ]
    );
    assert_eq!(world.models.len(), 1);
    assert_eq!(world.frames.len(), 1);
    assert_eq!(world.frames[0].name, "_merged__a__model__");
    assert_eq!(world.frames[0].attached_to, "a_child::link");
    assert_eq!(world.merged_includes.len(), 1);
}
