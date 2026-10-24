use sdformat::*;

#[test]
fn legacy_undeclared_prefixes_and_opaque_entities_survive_unchanged() {
    let plugin = "<plugin name='p' filename='driver.so'><custom:config custom:key='a &amp; b'>&bogus; &#0; <![CDATA[<raw/>]]></custom:config></plugin>";
    let xml = format!("<sdf version='1.12'><model name='m'><link name='l'/>{plugin}</model></sdf>");
    let sdf = load_str(&xml, "legacy.sdf", &NoResources).unwrap();
    assert_eq!(sdf.models[0].plugins[0].xml, plugin);
    assert_eq!(sdf.models[0].plugins[0].filename, "driver.so");
    let error = load_str(
        include_str!("fixtures/sensors.sdf"),
        "sensors.sdf",
        &NoResources,
    )
    .unwrap_err();
    assert_eq!(
        error.code,
        ErrorCode::Unsupported,
        "full original fixture must reach semantic validation: {error}"
    );
}

#[test]
fn references_cdata_comments_and_utf8_keep_typed_values_and_source_ranges() {
    let xml = "<?xml version='1.0'?><?loader policy='inert'?>\n<!-- source é -->\n<sdf version='1.12'><model name='m&amp;n'><link name='base&#x31;'><inertial><mass><![CDATA[2]]></mass></inertial><plugin name='p&#50;' filename='a&amp;b.so'><n:ü n:attr='&unknown;'/></plugin></link></model></sdf>";
    let sdf = load_str(xml, "unicode.sdf", &NoResources).unwrap();
    let m = &sdf.models[0];
    let link = &m.links[0];
    assert_eq!(m.name, "m&n");
    assert_eq!(link.name, "base1");
    assert_eq!(link.inertial.mass, 2.);
    assert_eq!(link.plugins[0].name, "p2");
    assert_eq!(link.plugins[0].filename, "a&b.so");
    assert_eq!(m.source.line, 3);
    assert_eq!(m.source.column, 21);
    assert_eq!(
        link.plugins[0].xml,
        "<plugin name='p&#50;' filename='a&amp;b.so'><n:ü n:attr='&unknown;'/></plugin>"
    );
    let xml = "<sdf version='1.12' xmlns:x='urn:test'><model name='a&amp;&bogus;'><link name='l'><inertial><mass>&#50;</mass></inertial></link></model></sdf>";
    let sdf = load_str(xml, "mixed", &NoResources).unwrap();
    assert_eq!(sdf.models[0].name, "a&&bogus;");
    assert_eq!(sdf.models[0].links[0].inertial.mass, 2.);
    let xml =
        "<sdf version='1.12'><model name='literal&amp;dangling&'><link name='l'/></model></sdf>";
    assert_eq!(
        load_str(xml, "literal", &NoResources).unwrap().models[0].name,
        "literal&dangling&"
    );
}

#[test]
fn malformed_xml_is_rejected_before_semantic_validation() {
    for xml in [
        "",
        " ",
        "<!-- only comment -->",
        "<sdf version='1.12'><model></sdf>",
        "<sdf version='1.12'><model",
        "<sdf version='1.12'><model name='m></model></sdf>",
        "<sdf version='1.12'/><other/>",
        "text<sdf version='1.12'/>",
        "<sdf version='1.12'/>text",
        "<![CDATA[text]]><sdf version='1.12'/>",
        "&amp;<sdf version='1.12'/>",
        "<sdf version='1.12'><model name='m' name='n'/></sdf>",
        "<sdf version='1.12'><model name='m'><link name='l'/><plugin name='p' filename='p'><1bad/></plugin></model></sdf>",
        "<sdf version='1.12'><model name='m'><link name='l'/><plugin name='p' filename='p' 1bad='v'/></model></sdf>",
        "<sdf version='1.12'><model name='m'><link name='l'/><plugin name='p' filename='p'><bad!/></plugin></model></sdf>",
        "<sdf version='1.12'><model name='m'><link name='l'/></model></sdf></sdf>",
        "<sdf version='1.12'><model name='m'><link name='l'/><plugin name='p' filename='p'>nul\0text</plugin></model></sdf>",
    ] {
        assert_eq!(
            load_str(xml, "malformed.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Xml,
            "{xml}"
        );
    }
    let xml = "<!DOCTYPE sdf [<!ENTITY local 'text'>]><sdf version='1.12'><model name='m'><link name='l'/></model></sdf>";
    assert_eq!(
        load_str(xml, "doctype", &NoResources).unwrap_err().code,
        ErrorCode::Unsupported
    );
}

#[test]
fn opaque_legacy_ampersands_are_not_treated_as_missing_entities() {
    for text in ["a& b", "&bare", "&#bad;", "&#0;", "&unknown;"] {
        let plugin = format!("<plugin name='p' filename='p'>{text}</plugin>");
        let xml =
            format!("<sdf version='1.12'><model name='m'><link name='l'/>{plugin}</model></sdf>");
        assert_eq!(
            load_str(&xml, "legacy-entity", &NoResources)
                .unwrap()
                .models[0]
                .plugins[0]
                .xml,
            plugin
        );
    }
}

#[test]
fn native_first_text_comments_and_cdata_contract() {
    for (content, expected) in [
        ("1<!-- comment -->2", 1.),
        ("<![CDATA[2]]>3", 2.),
        ("<!-- comment -->2", 2.),
        (" <!-- comment -->2", 2.),
        ("<![CDATA[]]>2", 1.),
        ("1&#50;3", 123.),
        ("1<![CDATA[2]]>3", 1.),
        ("<![CDATA[2]]><![CDATA[3]]>", 2.),
        ("2&#0;3", 2.),
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><inertial><mass>{content}</mass></inertial></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "first-text", &NoResources).unwrap().models[0].links[0]
                .inertial
                .mass,
            expected,
            "{content}"
        );
    }
    let plugin = "<plugin name='p' filename='p'><!-- invalid -- comment --></plugin>";
    let xml = format!("<sdf version='1.12'><model name='m'><link name='l'/>{plugin}</model></sdf>");
    assert_eq!(
        load_str(&xml, "legacy-comment", &NoResources)
            .unwrap()
            .models[0]
            .plugins[0]
            .xml,
        plugin
    );
}
#[test]
fn native_declaration_placement_and_recursive_depth_contract() {
    for declaration in ["<?xml version='1.0'?>", "<?1bad?>"] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'/><plugin name='p' filename='p'>{declaration}</plugin></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "nested-decl", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Xml
        );
    }
    for prefix in ["<?xml?>", "<?anything?>", "<?1bad?>"] {
        let xml =
            format!("{prefix}<sdf version='1.12'><model name='m'><link name='l'/></model></sdf>");
        assert!(load_str(&xml, "prolog", &NoResources).is_ok());
    }
    for (depth, accepted) in [(495, true), (496, false), (600, false)] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'/><plugin name='p' filename='p'>{}</plugin></model></sdf>",
            "<n>".repeat(depth) + &"</n>".repeat(depth)
        );
        assert_eq!(
            load_str(&xml, "depth", &NoResources).is_ok(),
            accepted,
            "{depth}"
        );
    }
}
#[test]
fn typed_numeric_control_references_follow_native_c_string_views() {
    for reference in ["&#0;", "&#00;", "&#x0;", "&#x0000;"] {
        let plugin = format!("<plugin name='p{reference}q' filename='a{reference}b'/>");
        let xml = format!(
            "<sdf version='1.12'><model name='m{reference}n'><link name='l'/>{plugin}</model></sdf>"
        );
        let sdf = load_str(&xml, "numeric-nul", &NoResources).unwrap();
        assert_eq!(sdf.models[0].name, "m");
        assert_eq!(sdf.models[0].plugins[0].name, "p");
        assert_eq!(sdf.models[0].plugins[0].filename, "a");
        assert_eq!(sdf.models[0].plugins[0].xml, plugin);
    }
}
