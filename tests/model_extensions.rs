// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

#[test]
fn opaque_model_children_keep_xml_attributes_order_and_owned_source() {
    let xml = r#"<sdf version="1.12"><model name="m"><link name="l"/>
<enable_wind>true</enable_wind>
<custom:foo name="first" a="x&amp;y"> A <![CDATA[B < C]]><!--keep--><child n="q"/> tail </custom:foo>
<custom:foo name=""/><unknown_element name="test_value"/></model></sdf>"#;
    let sdf = load_str(xml, "mem://extensions.sdf", &NoResources).unwrap();
    let model = &sdf.models[0];
    assert!(model.enable_wind);
    // Native DOM getters keep model and link flags independent. This does not
    // compute the effective wind that a physical consumer would apply.
    assert!(!model.links[0].properties.enable_wind);
    assert_eq!(model.extensions.len(), 3);
    let first = &model.extensions[0];
    assert_eq!(first.tag, "custom:foo");
    assert_eq!(first.attribute("name"), Some("first"));
    assert_eq!(first.attribute("a"), Some("x&y"));
    assert_eq!(first.attribute("missing"), None);
    assert_eq!(
        first
            .attributes
            .iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<_>>(),
        ["name", "a"]
    );
    assert_eq!(
        first.xml,
        r#"<custom:foo name="first" a="x&amp;y"> A <![CDATA[B < C]]><!--keep--><child n="q"/> tail </custom:foo>"#
    );
    assert_eq!(first.source.uri, "mem://extensions.sdf");
    assert_eq!(first.source.line, 3);
    assert_eq!(first.source.column, 1);
    assert_eq!(model.extensions[1].attribute("name"), Some(""));
    assert_eq!(model.extensions[2].tag, "unknown_element");
    let mut copied = model.clone();
    assert!(copied.enable_wind);
    copied.extensions[0].attributes[0].1 = "changed".into();
    copied.extensions[0].xml.clear();
    assert_eq!(model.extensions[0].attribute("name"), Some("first"));
    assert!(model.extensions[0].xml.contains("CDATA"));
    let retained = model.extensions.clone();
    drop(sdf);
    assert_eq!(retained[0].attribute("a"), Some("x&y"));
    assert!(retained[0].xml.contains("<!--keep-->"));
}

struct Child;
impl ResourceResolver for Child {
    fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
        Ok(Resource {
            canonical_uri: "resolved://child/model.sdf".into(),
            xml: r#"<sdf version="1.12"><model name="child"><link name="l"/><enable_wind>true</enable_wind><custom:foo name="baz" note="__model__">__model__</custom:foo><custom:bar/><unknown_element name="not_copied"/></model></sdf>"#.into(),
        })
    }
}
#[test]
fn merged_extensions_copy_colon_tags_without_rewriting_opaque_payload() {
    let xml = r#"<sdf version="1.12"><model name="parent"><link name="p"/><custom:local/><include merge="true"><uri>child</uri></include></model></sdf>"#;
    let sdf = load_str(xml, "root.sdf", &Child).unwrap();
    let model = &sdf.models[0];
    assert!(!model.enable_wind, "imported model wind flag is not merged");
    assert_eq!(model.links.len(), 2);
    assert!(model.links.iter().all(|l| !l.properties.enable_wind));
    assert_eq!(
        model
            .extensions
            .iter()
            .map(|e| e.tag.as_str())
            .collect::<Vec<_>>(),
        ["custom:local", "custom:foo", "custom:bar"]
    );
    assert_eq!(model.extensions[0].source.uri, "root.sdf");
    assert_eq!(model.extensions[1].source.uri, "resolved://child/model.sdf");
    assert_eq!(model.extensions[1].attribute("name"), Some("baz"));
    assert_eq!(model.extensions[1].attribute("note"), Some("__model__"));
    assert_eq!(
        model.extensions[1].xml,
        r#"<custom:foo name="baz" note="__model__">__model__</custom:foo>"#
    );
    assert_eq!(model.extensions[2].attribute("name"), None);
    assert_eq!(model.extensions[2].xml, "<custom:bar/>");
    assert!(!model.extensions.iter().any(|e| e.tag == "unknown_element"));
    let nested = load_str(r#"<sdf version="1.12"><model name="parent"><link name="p"/><include><uri>child</uri></include></model></sdf>"#, "root.sdf", &Child).unwrap();
    let child = &nested.models[0].models[0];
    assert!(child.enable_wind);
    assert_eq!(child.extensions.len(), 3);
    assert_eq!(child.extensions[2].tag, "unknown_element");
}

#[test]
fn known_model_schema_is_not_disguised_as_opaque_extensions() {
    for name in [
        "self_collide",
        "allow_auto_disable",
        "gripper",
        "model_state",
    ] {
        let xml =
            format!("<sdf version='1.12'><model name='m'><link name='l'/><{name}/></model></sdf>");
        assert_eq!(
            load_str(&xml, "root.sdf", &NoResources).unwrap_err().code,
            ErrorCode::Unsupported,
            "{name}"
        );
    }
    let xml = "<sdf version='1.12'><model name='m'><link name='l'/><enable_wind>maybe</enable_wind></model></sdf>";
    assert_eq!(
        load_str(xml, "root.sdf", &NoResources).unwrap_err().code,
        ErrorCode::InvalidValue
    );
    let xml = "<sdf version='1.12'><model name='m'><link name='l'/><enable_wind>true</enable_wind><enable_wind>false</enable_wind></model></sdf>";
    assert_eq!(
        load_str(xml, "root.sdf", &NoResources).unwrap_err().code,
        ErrorCode::InvalidValue
    );
}

#[test]
fn world_custom_merge_stays_explicitly_unqualified_instead_of_losing_data() {
    struct Nested;
    impl ResourceResolver for Nested {
        fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
            Ok(Resource { canonical_uri:"nested.sdf".into(),xml:"<sdf version='1.12'><model name='c'><model name='n'><link name='l'/></model><custom:foo/></model></sdf>".into() })
        }
    }
    let xml = "<sdf version='1.12'><world name='w'><include merge='true'><uri>nested</uri></include></world></sdf>";
    assert_eq!(
        load_str(xml, "root.sdf", &Nested).unwrap_err().code,
        ErrorCode::Unsupported
    );
}
