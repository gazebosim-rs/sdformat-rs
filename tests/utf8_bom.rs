// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{NoResources, load_str};
#[test]
fn bom_keeps_exact_opaque_and_configuration_spans_with_unicode_model_names() {
    let plugin = "<plugin name='p' filename='f'><data>α😀</data></plugin>";
    let configuration = "<force_torque><frame>sensor</frame></force_torque>";
    for name in ["m", "α", "😀"] {
        let xml = format!(
            "\u{feff}<sdf version='1.12'><model name='{name}'>{plugin}<link name='l'><sensor name='s' type='force_torque'>{configuration}</sensor></link></model></sdf>"
        );
        let document = load_str(&xml, "bom.sdf", &NoResources).unwrap();
        let model = &document.models[0];
        assert_eq!(model.plugins[0].xml, plugin);
        assert_eq!(model.links[0].sensors[0].configuration_xml, configuration);
    }
}
#[test]
fn bom_does_not_panic_on_a_multibyte_scalar_immediately_before_a_raw_subtree() {
    let xml = "\u{feff}<sdf version='1.12'><model name='😀'><plugin name='p' filename='f'/><link name='l'/></model></sdf>";
    let loaded = load_str(xml, "unicode-bom.sdf", &NoResources).unwrap();
    assert_eq!(
        loaded.models[0].plugins[0].xml,
        "<plugin name='p' filename='f'/>"
    );
}

fn unhex(value: &str) -> String {
    if value == "-" {
        return String::new();
    }
    String::from_utf8(
        (0..value.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
            .collect(),
    )
    .unwrap()
}
#[test]
fn actual_native_bom_corpus_preserves_typed_values_and_all_declared_raw_spans() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("utf8_bom.json")).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 87);
    for row in rows.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        let tokens = row["tokens"].as_array().unwrap();
        assert_eq!(tokens[0], "0", "{id}");
        let result = load_str(xml, "bom.sdf", &NoResources);
        if row.get("gap").is_some() {
            assert!(matches!(
                id,
                "grammar_gap_0" | "grammar_gap_1" | "grammar_gap_2"
            ));
            assert_eq!(result.unwrap_err().code, sdformat::ErrorCode::Xml, "{id}");
            continue;
        }
        let sdf = result.unwrap_or_else(|e| panic!("{id}: {e}"));
        let model = sdf
            .models
            .first()
            .unwrap_or_else(|| &sdf.worlds[0].models[0]);
        let mut expected = tokens.iter().skip(1).map(|v| v.as_str().unwrap());
        assert_eq!(model.name, unhex(expected.next().unwrap()), "{id}");
        assert_eq!(
            model.plugins.len(),
            expected.next().unwrap().parse::<usize>().unwrap(),
            "{id}"
        );
        for plugin in &model.plugins {
            assert_eq!(plugin.name, unhex(expected.next().unwrap()), "{id}");
            assert_eq!(plugin.filename, unhex(expected.next().unwrap()), "{id}");
            assert_eq!(plugin.xml, row["plugin_xml"].as_str().unwrap(), "{id}");
        }
        assert_eq!(
            model.links.len(),
            expected.next().unwrap().parse::<usize>().unwrap(),
            "{id}"
        );
        let link = &model.links[0];
        assert_eq!(link.name, unhex(expected.next().unwrap()), "{id}");
        assert_eq!(
            link.sensors.len(),
            expected.next().unwrap().parse::<usize>().unwrap(),
            "{id}"
        );
        let sensor = &link.sensors[0];
        assert_eq!(sensor.name, unhex(expected.next().unwrap()), "{id}");
        assert_eq!(sensor.kind, unhex(expected.next().unwrap()), "{id}");
        assert_eq!(
            sensor.force_torque().is_some(),
            expected.next().unwrap() == "1",
            "{id}"
        );
        assert_eq!(
            sensor.force_torque().unwrap().frame,
            sdformat::ForceTorqueFrame::Sensor,
            "{id}"
        );
        assert_eq!(expected.next(), Some("3"), "{id}");
        assert_eq!(
            sensor.force_torque().unwrap().measure_direction,
            sdformat::ForceTorqueMeasureDirection::ParentToChild,
            "{id}"
        );
        assert_eq!(expected.next(), Some("1"), "{id}");
        assert_eq!(
            sensor.configuration_xml,
            row["config_xml"].as_str().unwrap(),
            "{id}"
        );
        assert_eq!(sensor.topic, unhex(expected.next().unwrap()), "{id}");
        assert_eq!(
            model.joints.len(),
            expected.next().unwrap().parse::<usize>().unwrap(),
            "{id}"
        );
        let axis = model.joints[0].axis.as_ref();
        assert_eq!(axis.is_some(), expected.next().unwrap() == "1", "{id}");
        let axis = axis.unwrap();
        for value in [axis.dynamics.damping, axis.dynamics.friction] {
            assert_eq!(
                value.to_bits(),
                expected.next().unwrap().parse::<u64>().unwrap(),
                "{id}"
            );
        }
        assert_eq!(
            axis.dynamics.xml.as_deref(),
            row["axis_xml"].as_str(),
            "{id}"
        );
        assert_eq!(
            sdf.worlds.len(),
            expected.next().unwrap().parse::<usize>().unwrap(),
            "{id}"
        );
        if let Some(world) = sdf.worlds.first() {
            assert_eq!(
                world.physics[0].max_step_size.to_bits(),
                expected.next().unwrap().parse::<u64>().unwrap(),
                "{id}"
            );
            assert_eq!(
                world.physics[0].xml.as_deref(),
                row["physics_xml"].as_str(),
                "{id}"
            );
        }
        assert!(expected.next().is_none(), "{id}");
        for (source, needle) in [
            (&model.source, "<model name="),
            (&link.source, "<link name="),
            (&sensor.source, "<sensor name="),
            (
                sensor.force_torque().unwrap().source.as_ref().unwrap(),
                "<force_torque>",
            ),
        ] {
            let pos = xmlparser::Stream::from(xml).gen_text_pos_from(xml.find(needle).unwrap());
            assert_eq!((source.line, source.column), (pos.row, pos.col), "{id}");
        }
    }
}
#[test]
fn bom_source_coordinates_and_tokenizer_errors_use_original_document_offsets() {
    for body in [
        "<sdf version='1.12'><model name='m'><link name='l'/><frame name='f' attached_to='missing'/></model></sdf>",
        "<sdf version='1.12'><model name='m'><link name='l'></wrong></model></sdf>",
        "<sdf version='1.12'><model name='m'><link name='l' name='duplicate'/></model></sdf>",
    ] {
        for prefix in ["", "<?xml version='1.0'?>\r\n"] {
            let plain = format!("{prefix}{body}");
            let bom = format!("\u{feff}{plain}");
            let original = load_str(&plain, "bom.sdf", &NoResources).unwrap_err();
            let fixed = load_str(&bom, "bom.sdf", &NoResources).unwrap_err();
            assert_eq!(fixed.code, original.code);
            assert_eq!(fixed.message, original.message);
            assert_eq!(fixed.source.uri, original.source.uri);
            assert_eq!(fixed.source.line, original.source.line);
            assert_eq!(
                fixed.source.column,
                original.source.column + u32::from(original.source.line == 1)
            );
        }
    }
    let config = "\u{feff}<model><name>😀</name><sdf version='99'>future.sdf</sdf></model>";
    let selected = sdformat::select_model_config(config, "config").unwrap();
    let pos = xmlparser::Stream::from(config).gen_text_pos_from(config.find("<sdf").unwrap());
    assert_eq!(
        (
            selected.warnings[0].source.line,
            selected.warnings[0].source.column
        ),
        (pos.row, pos.col)
    );
}
#[test]
fn bom_include_and_resource_configuration_keep_distinct_original_spans() {
    struct Resolver(String);
    impl sdformat::ResourceResolver for Resolver {
        fn resolve(&self, _: &str, _: &str) -> Result<sdformat::Resource, String> {
            Ok(sdformat::Resource {
                canonical_uri: "resolved://😀".into(),
                xml: self.0.clone(),
            })
        }
    }
    let config = "<force_torque><frame>sensor</frame></force_torque>";
    let resource = format!(
        "\u{feff}<sdf version='1.12'><model name='😀'><link name='l'><sensor name='s' type='force_torque'>{config}</sensor></link></model></sdf>"
    );
    let include = "<include><uri>resource</uri></include>";
    let root = format!("\u{feff}<sdf version='1.12'><model name='parent'>{include}</model></sdf>");
    let loaded = load_str(&root, "parent", &Resolver(resource)).unwrap();
    let model = &loaded.models[0].models[0];
    assert_eq!(model.include.as_ref().unwrap().xml, include);
    assert_eq!(model.links[0].sensors[0].configuration_xml, config);
    assert_eq!(model.links[0].sensors[0].source.uri, "resolved://😀");
}

#[test]
fn bom_diagnostic_after_unicode_model_name_returns_error_without_utf8_panic() {
    let xml = "\u{feff}<sdf version='1.12'><model name='😀'><link name='l' unknown='value'/></model></sdf>";
    let error = load_str(xml, "bom.sdf", &NoResources).unwrap_err();
    assert_eq!(error.code, sdformat::ErrorCode::Unsupported);
    let pos = xmlparser::Stream::from(xml).gen_text_pos_from(xml.find("<link").unwrap());
    assert_eq!((error.source.line, error.source.column), (pos.row, pos.col));
}
