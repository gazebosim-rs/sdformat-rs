// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Native namespace projections are supplemental, not additional original ports.
use sdformat::{ErrorCode, Model, NoResources, Resource, ResourceResolver, load_str};
use serde_json::{Value, json};
use std::cell::Cell;

struct Resolver<'a> {
    xml: Option<&'a str>,
    calls: Cell<usize>,
}
impl ResourceResolver for Resolver<'_> {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        assert_eq!(uri, "model://namespace-case");
        assert_eq!(parent, "namespace.xml");
        self.calls.set(self.calls.get() + 1);
        Ok(Resource {
            canonical_uri: "resolved://namespace.sdf".into(),
            xml: self.xml.ok_or("unexpected resource request")?.into(),
        })
    }
}
fn projection(model: &Model) -> Value {
    json!({
        "name": model.name,
        "raw_namespace": model.raw_namespace,
        "namespace": model.namespace(),
        "links": model.links.len(),
        "frames": model.frames.iter().map(|f| &f.name).collect::<Vec<_>>(),
        "models": model.models.iter().map(projection).collect::<Vec<_>>()
    })
}
fn check(prefixes: &[&str], count: usize) {
    let corpus: Value = serde_json::from_str(include_str!("model_namespace_xml.json")).unwrap();
    assert_eq!(corpus.as_array().unwrap().len(), 98);
    let mut seen = 0;
    for case in corpus.as_array().unwrap().iter().filter(|c| {
        prefixes
            .iter()
            .any(|p| c["id"].as_str().unwrap().starts_with(p))
    }) {
        seen += 1;
        let resolver = Resolver {
            xml: case["resource"].as_str(),
            calls: Cell::new(0),
        };
        let xml = case["xml"]
            .as_str()
            .unwrap()
            .replace("@RESOURCE@", "model://namespace-case");
        let sdf = load_str(&xml, "namespace.xml", &resolver)
            .unwrap_or_else(|error| panic!("{}: {error}", case["id"]));
        assert!(sdf.worlds.is_empty());
        assert_eq!(sdf.models.len(), 1);
        assert_eq!(
            projection(&sdf.models[0]),
            case["expected"],
            "{}",
            case["id"]
        );
        assert_eq!(
            resolver.calls.get(),
            usize::from(case.get("resource").is_some())
        );
        if case["id"].as_str().unwrap().starts_with("include-") {
            let included = &sdf.models[0].models[0];
            assert_eq!(included.source.uri, "resolved://namespace.sdf");
            let include = included.include.as_ref().unwrap();
            assert_eq!(include.uri, "model://namespace-case");
            assert_eq!(include.source.uri, "namespace.xml");
            assert!(xml.contains(&include.xml));
        }
    }
    assert_eq!(seen, count);
}
#[test]
fn native_direct_namespace_values_across_nine_versions() {
    check(&["direct-"], 36);
}
#[test]
fn native_include_namespace_overrides_empty_cdata_and_rename() {
    check(&["include-"], 56);
}
#[test]
fn native_nested_and_merged_namespace_values_do_not_inherit() {
    check(&["nested-", "merge-"], 6);
}

#[test]
fn malformed_namespace_children_attributes_and_duplicates_are_rejected() {
    for (body, code) in [
        ("<namespace bad='x'>ns</namespace>", ErrorCode::Unsupported),
        ("<namespace><bad/></namespace>", ErrorCode::Unsupported),
        (
            "<namespace>a</namespace><namespace>b</namespace>",
            ErrorCode::InvalidValue,
        ),
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='holder'><include><uri>model://namespace-case</uri>{body}</include></model></sdf>"
        );
        let resolver = Resolver {
            xml: Some("<sdf version='1.12'><model name='child'><link name='base'/></model></sdf>"),
            calls: Cell::new(0),
        };
        let error = load_str(&xml, "namespace.xml", &resolver).unwrap_err();
        assert_eq!(error.code, code);
        assert!(error.message.contains("namespace") || error.message.contains("bad"));
    }
}

#[test]
fn world_namespace_trims_only_native_space_tab_newline_and_keeps_inner_text() {
    let xml = "<sdf version='1.12'><world name='w'><model name='robot' namespace='  ns/{name}  tail  '><link name='base'/></model></world></sdf>";
    let sdf = load_str(xml, "world-namespace.sdf", &NoResources).unwrap();
    let model = &sdf.worlds[0].models[0];
    assert_eq!(model.raw_namespace.as_deref(), Some("ns/{name}  tail"));
    assert_eq!(model.namespace().as_deref(), Some("ns/robot  tail"));
    assert_eq!(model.scoped_name, "w::robot");
    let unicode = "<sdf version='1.12'><model name='robot' namespace='\u{a0}{name}\u{a0}'><link name='base'/></model></sdf>";
    let sdf = load_str(unicode, "unicode-namespace.sdf", &NoResources).unwrap();
    assert_eq!(
        sdf.models[0].namespace().as_deref(),
        Some("\u{a0}robot\u{a0}")
    );
}
