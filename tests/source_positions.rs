// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, Source, load_str};
use xmlparser::Stream;
fn check(source: &Source, xml: &str, needle: &str) {
    let offset = xml.find(needle).unwrap();
    // The existing full-source routine is the independent compatibility oracle.
    let expected = Stream::from(xml).gen_text_pos_from(offset);
    assert_eq!(
        (source.line, source.column),
        (expected.row, expected.col),
        "{needle}: {xml}"
    );
    assert_eq!(source.uri, "positions://α😀");
}
fn input(prefix: &str, separator: &str, opaque: &str) -> String {
    format!(
        "{prefix}<sdf version='1.12'>{separator}<model name='m'>{separator}<plugin name='p' filename='f'>{opaque}</plugin>{separator}<link name='l'>{separator}<collision name='c'><geometry><box><size>1 1 1</size></box></geometry></collision>{separator}<sensor name='ft' type='force_torque'><force_torque><frame>child</frame></force_torque></sensor>{separator}</link>{separator}<frame name='f'/>{separator}<joint name='j' type='fixed'><parent>world</parent><child>l</child></joint>{separator}</model></sdf>"
    )
}
#[test]
fn public_source_positions_match_old_full_prefix_oracle_across_raw_unicode_and_eols() {
    for prefix in ["", "\u{feff}", "<?xml version='1.0'?>\r\n<!--α😀-->\n"] {
        for separator in [
            "",
            " ",
            "\n",
            "\r",
            "\r\n",
            "\n\r",
            "\t\n\t",
            "<!--\u{85}\u{2028}\u{2029}😀-->",
        ] {
            for opaque in [
                "",
                "<x/>",
                "<x attr='α&#10;😀'><y>λ\n\r\t😀</y></x>",
                "<!--α\r\n😀--><x><![CDATA[\n\r😀&#10;]]></x><y a='&#13;&#10;' />",
            ] {
                let xml = input(prefix, separator, opaque);
                let sdf = load_str(&xml, "positions://α😀", &NoResources).unwrap();
                let model = &sdf.models[0];
                let link = &model.links[0];
                for (source, needle) in [
                    (&model.source, "<model name='m'>"),
                    (&link.source, "<link name='l'>"),
                    (&link.collisions[0].source, "<collision name='c'>"),
                    (&link.sensors[0].source, "<sensor name='ft'"),
                    (
                        link.sensors[0]
                            .force_torque()
                            .unwrap()
                            .source
                            .as_ref()
                            .unwrap(),
                        "<force_torque>",
                    ),
                    (&model.frames[0].source, "<frame name='f'/>"),
                    (&model.joints[0].source, "<joint name='j'"),
                ] {
                    check(source, &xml, needle);
                }
            }
        }
    }
}
#[test]
fn diagnostic_leaf_and_graph_positions_match_old_full_prefix_oracle() {
    for separator in ["", "\n", "\r\n", "<!--😀\rα\nλ-->"] {
        let xml = input(
            "<?xml version='1.0'?>\n",
            separator,
            "<ignored>😀α</ignored>",
        );
        for (old, new, needle, code) in [
            (
                "<size>1 1 1</size>",
                "<size>bad</size>",
                "<size>",
                ErrorCode::InvalidValue,
            ),
            (
                "<frame>child</frame>",
                "<frame>BAD</frame>",
                "<frame>",
                ErrorCode::InvalidValue,
            ),
            (
                "<frame name='f'/>",
                "<frame name='f' attached_to='missing'/>",
                "<frame name='f'",
                ErrorCode::UnknownReference,
            ),
        ] {
            let broken = xml.replace(old, new);
            let error = load_str(&broken, "positions://α😀", &NoResources).unwrap_err();
            assert_eq!(error.code, code);
            check(&error.source, &broken, needle);
        }
    }
}
#[test]
fn imported_sources_keep_resource_local_positions_and_raw_configuration() {
    struct Resolver(String);
    impl sdformat::ResourceResolver for Resolver {
        fn resolve(&self, _: &str, _: &str) -> Result<sdformat::Resource, String> {
            Ok(sdformat::Resource {
                canonical_uri: "positions://α😀".into(),
                xml: self.0.clone(),
            })
        }
    }
    let raw = "<force_torque><frame>child</frame></force_torque>";
    let xml = input("<!--😀α-->\n", "\r\n", "<x>α</x>");
    let root = "<sdf version='1.12'><model name='parent'><include><uri>resource</uri></include></model></sdf>";
    let sdf = load_str(root, "parent.sdf", &Resolver(xml.clone())).unwrap();
    let model = &sdf.models[0].models[0];
    check(&model.source, &xml, "<model name='m'>");
    let sensor = &model.links[0].sensors[0];
    check(&sensor.source, &xml, "<sensor name='ft'");
    assert_eq!(sensor.configuration_xml, raw);
}
