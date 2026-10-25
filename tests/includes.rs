// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Copyright 2018 Open Source Robotics Foundation
// Parsing assertions from DOMModel.IncludeModelWithPlugin; export assertions remain pending.
use sdformat::*;
use std::cell::RefCell;

struct Resolver {
    xml: &'static str,
    requests: RefCell<Vec<(String, String)>>,
}
impl ResourceResolver for Resolver {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        self.requests.borrow_mut().push((uri.into(), parent.into()));
        Ok(Resource {
            canonical_uri: "resolved://resource.sdf".into(),
            xml: self.xml.into(),
        })
    }
}
fn resolver() -> Resolver {
    Resolver {
        xml: include_str!("fixtures/include_resource.sdf"),
        requests: RefCell::default(),
    }
}
fn plugins(plugins: &[Plugin]) -> Vec<[String; 2]> {
    plugins
        .iter()
        .map(|plugin| [plugin.name.clone(), plugin.filename.clone()])
        .collect()
}
#[test]
fn native_include_metadata_and_callback_text() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("include.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 61);
    for case in cases.as_array().unwrap() {
        let resolver = Resolver {
            xml: if case["resource"] == "test_model_with_plugin.sdf" {
                include_str!("fixtures/test_model_with_plugin.sdf")
            } else {
                include_str!("fixtures/include_resource.sdf")
            },
            requests: RefCell::default(),
        };
        let xml = case["xml"].as_str().unwrap();
        let result = load_str(xml, "parent.sdf", &resolver);
        let expected = &case["expected"];
        if expected.get("exception").is_some() {
            // Keep the previously defined safe defaults; native throws on null
            // text here. These rows are explicit extensions, not parity passes.
            let sdf = result.unwrap();
            let model = &sdf.models[0].models[0];
            let include = model.include.as_ref().unwrap();
            if case["id"] == "static_empty" {
                assert!(!model.is_static);
                assert_eq!(include.is_static, Some(false));
            } else {
                assert_eq!(model.placement_frame, "");
                assert_eq!(include.placement_frame.as_deref(), Some(""));
            }
            continue;
        }
        assert_eq!(
            result.is_ok(),
            expected["accepted"].as_bool().unwrap(),
            "{case}"
        );
        if !expected["accepted"].as_bool().unwrap() {
            continue;
        }
        let sdf = result.unwrap();
        let outer = sdf
            .models
            .first()
            .unwrap_or_else(|| &sdf.worlds[0].models[0]);
        if !sdf.models.is_empty() {
            assert!(outer.include.is_none());
        }
        let model = outer.models.first().unwrap_or(outer);
        assert_eq!(model.name, expected["name"].as_str().unwrap(), "{case}");
        assert_eq!(
            model.is_static,
            expected["is_static"].as_bool().unwrap(),
            "{case}"
        );
        assert_eq!(
            serde_json::json!(plugins(&model.plugins)),
            expected["plugins"],
            "{case}"
        );
        let requests: Vec<_> = resolver
            .requests
            .borrow()
            .iter()
            .map(|(uri, parent)| {
                assert_eq!(parent, "parent.sdf");
                uri.clone()
            })
            .collect();
        assert_eq!(serde_json::json!(requests), expected["requests"], "{case}");
        match (&model.include, &expected["include"]) {
            (None, serde_json::Value::Null) => assert_eq!(model.source.uri, "parent.sdf"),
            (Some(include), expected) => {
                assert_eq!(model.source.uri, "resolved://resource.sdf");
                assert_eq!(include.source.uri, "parent.sdf");
                assert_eq!(include.uri, expected["uri"].as_str().unwrap());
                assert_eq!(include.name.as_deref(), expected["name"].as_str());
                assert_eq!(include.is_static, expected["is_static"].as_bool());
                assert_eq!(
                    include.placement_frame.as_deref(),
                    expected["placement_frame"].as_str()
                );
                assert_eq!(
                    serde_json::json!(plugins(&include.plugins)),
                    expected["plugins"]
                );
                let pose = include.pose.as_ref().map(|p| {
                    p.transform
                        .translation
                        .into_iter()
                        .chain(p.transform.rotation)
                        .map(f64::to_bits)
                        .collect::<Vec<_>>()
                });
                assert_eq!(serde_json::json!(pose), expected["pose_bits"]);
                assert_eq!(
                    include.pose.as_ref().map(|p| p.relative_to.as_str()),
                    expected["pose_relative_to"].as_str()
                );
                let start = xml.find("<include>").unwrap();
                let end = xml.rfind("</include>").unwrap() + "</include>".len();
                assert_eq!(include.xml, xml[start..end]);
            }
            _ => panic!("include presence mismatch: {case}"),
        }
    }
}

#[test]
fn original_include_model_plugins_parsing_subset() {
    let resolver = Resolver {
        xml: include_str!("fixtures/test_model_with_plugin.sdf"),
        requests: RefCell::default(),
    };
    let document = load_str(
        include_str!("fixtures/include_model_with_plugin.sdf"),
        "include_model_with_plugin.sdf",
        &resolver,
    )
    .unwrap();
    let world = document.worlds.first().unwrap();
    let model = world.models.first().unwrap();
    let include = model.include.as_ref().unwrap();
    assert_eq!(include.uri, "test_model_with_plugin");
    assert_eq!(include.plugins.len(), 1);
    assert_eq!(include.plugins[0].name, "test_model_include_plugin");
    assert_eq!(include.plugins[0].filename, "include/test/model/plugin");
    assert_eq!(model.plugins.len(), 2);
    assert_eq!(model.plugins[0].name, "test_model_plugin");
    assert_eq!(model.plugins[0].filename, "test/model/plugin");
    assert_eq!(model.plugins[1].name, "test_model_include_plugin");
    assert_eq!(model.plugins[1].filename, "include/test/model/plugin");
}

#[test]
fn include_values_are_owned_snapshots_and_errors_stay_explicit() {
    let resolver = resolver();
    let xml = "<sdf version='1.12'><model name='outer'><include><uri>r</uri><plugin name='p' filename='p.so'><x:arg>original</x:arg></plugin></include></model></sdf>";
    let mut sdf = load_str(xml, "parent.sdf", &resolver).unwrap();
    let model = &mut sdf.models[0].models[0];
    model.plugins[1].filename = "changed".into();
    let include = model.include.as_ref().unwrap();
    assert_eq!(include.plugins[0].filename, "p.so");
    assert!(include.plugins[0].xml.contains("<x:arg>original</x:arg>"));
    assert!(include.xml.contains("filename='p.so'"));
    let old = Resolver {
        xml: "<sdf version='1.5'><model name='legacy'><link name='l'/></model></sdf>",
        requests: RefCell::default(),
    };
    assert_eq!(
        load_str(
            "<sdf version='1.12'><model name='m'><include><uri>old</uri></include></model></sdf>",
            "parent.sdf",
            &old
        )
        .unwrap_err()
        .code,
        ErrorCode::Version
    );
    for (content, code) in [
        ("<include/>", ErrorCode::Missing),
        ("<include><uri/></include>", ErrorCode::Missing),
        ("<include><uri>r</uri><name/></include>", ErrorCode::Missing),
        (
            "<include merge='false'><uri>r</uri></include>",
            ErrorCode::Unsupported,
        ),
        (
            "<include><uri>r</uri><unknown/></include>",
            ErrorCode::Unsupported,
        ),
    ] {
        let xml = format!("<sdf version='1.12'><model name='outer'>{content}</model></sdf>");
        assert_eq!(
            load_str(&xml, "parent.sdf", &resolver).unwrap_err().code,
            code
        );
    }
}

#[test]
fn nested_include_sources_and_parent_resolution_identity_are_retained() {
    struct Nested(RefCell<Vec<(String, String)>>);
    impl ResourceResolver for Nested {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            self.0.borrow_mut().push((uri.into(), parent.into()));
            Ok(Resource {
                canonical_uri: format!("resolved://{uri}"),
                xml: if uri == "middle" {
                    "<sdf version='1.12'><model name='middle'><include><uri>leaf</uri></include></model></sdf>"
                } else {
                    "<sdf version='1.12'><model name='leaf'><link name='l'/></model></sdf>"
                }.into(),
            })
        }
    }
    let resolver = Nested(RefCell::default());
    let sdf=load_str("<sdf version='1.12'><model name='outer'><include><uri>middle</uri></include></model></sdf>","parent.sdf",&resolver).unwrap();
    let middle = &sdf.models[0].models[0];
    let leaf = &middle.models[0];
    assert_eq!(
        *resolver.0.borrow(),
        [
            ("middle".into(), "parent.sdf".into()),
            ("leaf".into(), "resolved://middle".into())
        ]
    );
    assert_eq!(middle.source.uri, "resolved://middle");
    assert_eq!(middle.include.as_ref().unwrap().source.uri, "parent.sdf");
    assert_eq!(leaf.source.uri, "resolved://leaf");
    assert_eq!(
        leaf.include.as_ref().unwrap().source.uri,
        "resolved://middle"
    );
    assert_eq!(leaf.include.as_ref().unwrap().uri, "leaf");
}
