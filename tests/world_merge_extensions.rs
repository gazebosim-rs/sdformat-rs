// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, Resource, ResourceResolver, load_str};
use serde_json::Value;

struct Resolver<'a>(&'a Value);
impl ResourceResolver for Resolver<'_> {
    fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
        let (key, canonical_uri) = match uri {
            "@A@" => ("resource_a", "import-a.sdf"),
            "@B@" => ("resource_b", "import-b.sdf"),
            _ => return Err(uri.into()),
        };
        Ok(Resource {
            canonical_uri: canonical_uri.into(),
            xml: self.0[key].as_str().unwrap().into(),
        })
    }
}

#[test]
fn qualified_world_merge_metadata_tracks_native_order_and_owned_origins() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("world_merge_extensions.json")).unwrap();
    assert_eq!(rows.len(), 14);
    for row in &rows {
        let id = row["id"].as_str().unwrap();
        let xml = row["world"].as_str().unwrap();
        if id == "direct_world" {
            assert_eq!(
                load_str(xml, "root.sdf", &Resolver(row)).unwrap_err().code,
                ErrorCode::Unsupported
            );
            continue;
        }
        let sdf = load_str(xml, "root.sdf", &Resolver(row)).unwrap();
        let world = &sdf.worlds[0];
        let native = row["stdout"].as_str().unwrap();
        assert!(
            native.starts_with(&format!(
                "models={} frames={}\n",
                world.models.len(),
                world.frames.len()
            )),
            "{id}"
        );
        let expected: &[(&str, &str, &str)] = if id.starts_with("two_") {
            &[
                ("v:alpha", "<v:alpha k='1'/>", "import-a.sdf"),
                ("v:alpha", "<v:alpha k='2'/>", "import-a.sdf"),
                ("v:beta", "<v:beta>text</v:beta>", "import-a.sdf"),
                (
                    "v:gamma",
                    "<v:gamma><child>payload</child></v:gamma>",
                    "import-b.sdf",
                ),
            ]
        } else if id == "old_rejection" {
            &[("custom:foo", "<custom:foo/>", "import-a.sdf")]
        } else if id == "nested" {
            &[
                ("v:inner", "<v:inner>I</v:inner>", "import-b.sdf"),
                ("v:outer", "<v:outer>O</v:outer>", "import-a.sdf"),
            ]
        } else if id == "multiline" {
            &[(
                "v:meta",
                "<v:meta key='α&amp;β'>π<v:child/>tail</v:meta>",
                "import-a.sdf",
            )]
        } else if id == "unqualified" {
            &[("v:keep", "<v:keep>yes</v:keep>", "import-a.sdf")]
        } else {
            &[]
        };
        assert_eq!(world.extensions.len(), expected.len(), "{id}");
        let mut native_position = 0;
        for (actual, (tag, original, uri)) in world.extensions.iter().zip(expected) {
            assert_eq!(&actual.tag, tag, "{id}");
            assert_eq!(&actual.xml, original, "{id}");
            assert_eq!(&actual.source.uri, uri, "{id}");
            let source = if *uri == "import-a.sdf" {
                &row["resource_a"]
            } else {
                &row["resource_b"]
            }
            .as_str()
            .unwrap();
            let prefix = &source[..source.find(original).unwrap()];
            let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
            let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
            assert_eq!(actual.source.line as usize, line);
            assert_eq!(actual.source.column as usize, column);
            let marker = format!("<{tag}");
            let offset = native[native_position..]
                .find(&marker)
                .unwrap_or_else(|| panic!("{id}: missing ordered {tag}"));
            native_position += offset + marker.len();
        }
        if id.starts_with("two_") {
            assert_eq!(world.extensions[0].attribute("k"), Some("1"));
            assert_eq!(world.extensions[1].attribute("k"), Some("2"));
        }
        if id == "multiline" {
            assert_eq!(world.extensions[0].attribute("key"), Some("α&β"));
        }
        if id == "unqualified" {
            assert!(!native.contains("<other>"));
        }
        assert_eq!(
            world.merged_includes.len(),
            if id.starts_with("two_") || id == "nested" {
                2
            } else {
                1
            }
        );
        let snapshot = world.extensions.clone();
        drop(sdf);
        assert_eq!(snapshot.len(), expected.len());
        for (actual, (_, original, _)) in snapshot.iter().zip(expected) {
            assert_eq!(&actual.xml, original);
        }
    }
}

#[test]
fn unrelated_world_and_merge_rejections_remain() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("world_merge_extensions.json")).unwrap();
    let row = &rows[0];
    let direct_link = serde_json::json!({"resource_a":"<sdf version='1.12'><model name='m'><link name='l'/><custom:foo/></model></sdf>"});
    let merge = "<sdf version='1.12'><world name='w'><include merge='true'><uri>@A@</uri></include></world></sdf>";
    assert_eq!(
        load_str(merge, "root.sdf", &Resolver(&direct_link))
            .unwrap_err()
            .code,
        ErrorCode::MergeIncludeUnsupported
    );
    for xml in [
        "<sdf version='1.12'><world name='w'><unknown/></world></sdf>",
        "<sdf version='1.12'><world name='w'><gravity>0 0 -9.8</gravity><gravity>0 0 -9.8</gravity></world></sdf>",
    ] {
        assert!(load_str(xml, "root.sdf", &Resolver(row)).is_err());
    }
}

#[test]
fn exact_prior_world_merge_input_retains_custom_metadata() {
    struct Nested;
    impl ResourceResolver for Nested {
        fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
            Ok(Resource { canonical_uri:"nested.sdf".into(),xml:"<sdf version='1.12'><model name='c'><model name='n'><link name='l'/></model><custom:foo/></model></sdf>".into() })
        }
    }
    let xml = "<sdf version='1.12'><world name='w'><include merge='true'><uri>nested</uri></include></world></sdf>";
    let sdf = load_str(xml, "root.sdf", &Nested).unwrap();
    let world = &sdf.worlds[0];
    assert_eq!(world.models.len(), 1);
    assert_eq!(world.models[0].name, "n");
    assert_eq!(world.frames.len(), 1);
    assert_eq!(world.frames[0].name, "_merged__c__model__");
    assert_eq!(world.frames[0].attached_to, "n::l");
    assert_eq!(
        sdf.pose_of("w::n::l", "w").unwrap(),
        sdformat::Transform::default()
    );
    assert_eq!(world.extensions.len(), 1);
    assert_eq!(world.extensions[0].tag, "custom:foo");
    assert_eq!(world.extensions[0].xml, "<custom:foo/>");
    assert_eq!(world.extensions[0].source.uri, "nested.sdf");
    let debug = format!("{world:?}");
    assert!(debug.contains("custom:foo"));
    assert!(debug.contains("nested.sdf"));
    assert_eq!(world.merged_includes.len(), 1);
}
