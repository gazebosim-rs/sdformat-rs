// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, Sensor, SensorType, load_str};
const FIXTURE: &str = include_str!("fixtures/environmental_sensor.sdf.in");
fn sensor(xml: &str) -> Result<Sensor, sdformat::Diagnostic> {
    let mut sensor = Sensor::default();
    sensor.load_element_str(xml, "custom.sdf", &NoResources)?;
    Ok(sensor)
}
#[test]
fn unchanged_native_environmental_world_retains_all_three_sensor_payloads() {
    let sdf = load_str(FIXTURE, "environmental_sensor.sdf.in", &NoResources).unwrap();
    let sensors = &sdf.worlds[0].models[1].links[0].sensors;
    assert_eq!(sensors.len(), 3);
    for (sensor, name, subtype, tags) in [
        (&sensors[0], "custom_sensor", "humidity", &[][..]),
        (
            &sensors[1],
            "wind_2d",
            "wind_speed2d",
            &[
                "output_format",
                "environment_variable_x",
                "environment_variable_y",
            ][..],
        ),
        (
            &sensors[2],
            "wind_3d",
            "wind_speed3d",
            &[
                "output_format",
                "transform_type",
                "environment_variable_x",
                "environment_variable_y",
                "environment_variable_z",
            ][..],
        ),
    ] {
        assert_eq!(sensor.name, name);
        assert_eq!(sensor.kind, "custom");
        assert_eq!(sensor.effective_type(), Some(SensorType::Custom));
        assert_eq!(sensor.update_rate, 30.);
        assert!(sensor.always_on && sensor.visualize);
        assert_eq!(
            sensor.custom().unwrap().custom_type.as_deref(),
            Some(format!("environmental_sensor/{subtype}").as_str())
        );
        assert_eq!(
            sensor
                .extensions
                .iter()
                .map(|e| e.tag.as_str())
                .collect::<Vec<_>>(),
            tags
        );
        for item in &sensor.extensions {
            assert_eq!(item.source.uri, "environmental_sensor.sdf.in");
            assert!(FIXTURE.contains(&item.xml));
            assert_eq!(
                FIXTURE
                    .lines()
                    .nth(item.source.line as usize - 1)
                    .unwrap()
                    .trim(),
                item.xml
            );
        }
        assert!(sensor.to_xml().is_err());
    }
    assert_eq!(sensors[1].topic, "wind_speed2d");
    assert_eq!(sensors[2].topic, "wind_speed3d");
    assert_eq!(
        sensors[2].extensions[1].xml,
        "<transform_type>LOCAL</transform_type>"
    );
}
#[test]
fn custom_subtype_presence_decoding_and_extension_order_are_owned() {
    assert_eq!(
        sensor("<sensor name='s' type='custom'/>")
            .unwrap()
            .custom()
            .unwrap()
            .custom_type,
        None
    );
    assert_eq!(
        sensor("<sensor name='s' type='custom' gz:type=''/>")
            .unwrap()
            .custom()
            .unwrap()
            .custom_type
            .as_deref(),
        Some("")
    );
    let s = sensor("<sensor name='s' type='custom' gz:type='environmental_sensor/wind_speed2d'><environment_variable_x k='a&amp;b'>one</environment_variable_x><output_format>vector3</output_format><environment_variable_x>two</environment_variable_x></sensor>").unwrap();
    assert_eq!(s.extensions.len(), 3);
    assert_eq!(s.extensions[0].attribute("k"), Some("a&b"));
    assert_eq!(
        s.extensions[2].xml,
        "<environment_variable_x>two</environment_variable_x>"
    );
    assert_eq!(s.extensions[0].source.uri, "custom.sdf");
    assert_eq!(s.extensions[0].source.line, 1);
    assert_eq!(s.extensions[0].source.column, 76);
    assert_eq!(
        sensor("<sensor name='s' type='custom' gz:type='a&amp;b'/>")
            .unwrap()
            .custom()
            .unwrap()
            .custom_type
            .as_deref(),
        Some("a&b")
    );
}
#[test]
fn subtype_does_not_widen_other_families_or_known_payloads() {
    for xml in [
        "<sensor name='s' type='custom'><imu/></sensor>",
        "<sensor name='s' type='imu' gz:type='environmental_sensor/humidity'/>",
        "<sensor name='s' type='custom' other:type='x'/>",
    ] {
        assert_eq!(sensor(xml).unwrap_err().code, ErrorCode::Unsupported);
    }
    assert_eq!(sensor("<sensor name='s' type='custom' gz:type='environmental_sensor/humidity'><always_on>1</always_on><always_on>0</always_on></sensor>").unwrap_err().code, ErrorCode::InvalidValue);
}

#[test]
fn arbitrary_custom_fields_keep_nested_raw_xml_and_duplicates_without_subtype() {
    let raw = "<custom_field a='one'><nested>value&amp;text</nested><!--note--><![CDATA[tail]]></custom_field>";
    for attr in ["", " gz:type=''", " gz:type='arbitrary/type'"] {
        let xml = format!(
            "<sensor name='s' type='custom'{attr}>{raw}<custom_field/><another:field/></sensor>"
        );
        let s = sensor(&xml).unwrap();
        assert_eq!(s.extensions.len(), 3);
        assert_eq!(s.extensions[0].tag, "custom_field");
        assert_eq!(s.extensions[0].xml, raw);
        assert_eq!(s.extensions[1].xml, "<custom_field/>");
        assert_eq!(s.extensions[2].xml, "<another:field/>");
        assert_eq!(s.configuration_xml, "<another:field/>");
        assert_eq!(s.custom().unwrap().element_name, "another:field");
        assert!(s.to_xml().is_err());
    }
}

#[test]
fn subtype_uses_native_param_edge_trim_while_raw_attribute_bytes_remain() {
    let xml = "<sensor name='s' type='custom' gz:type='  a  b  '/>";
    let s = sensor(xml).unwrap();
    let c = s.custom().unwrap();
    assert_eq!(c.custom_type.as_deref(), Some("a  b"));
    assert_eq!(c.xml, xml);
    assert!(sensor("<sensor name='s' type='custom' gz:type='a' gz:type='b'/>").is_err());
}

#[test]
fn native_matrix_preserves_effective_subtype_and_full_sensor_envelopes() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("custom_sensor_native.json")).unwrap();
    for row in corpus["observations"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        let loaded = load_str(xml, "matrix.sdf", &NoResources);
        if name == "known_payload_strict_difference" {
            assert_eq!(loaded.unwrap_err().code, ErrorCode::Unsupported);
            continue;
        }
        if row["exit"].as_i64().unwrap() != 0 {
            assert_eq!(loaded.unwrap_err().code, ErrorCode::Xml);
            continue;
        }
        let loaded = loaded.unwrap();
        let model = loaded.worlds[0]
            .models
            .iter()
            .find(|m| m.name == "model_with_sensor")
            .unwrap();
        let sensors = &model.links[0].sensors;
        let expected: Vec<_> = row["stdout"]
            .as_str()
            .unwrap()
            .split('\n')
            .filter(|s| s.starts_with("SENSOR ") || s.starts_with("CUSTOM_TYPE "))
            .collect();
        let actual: Vec<_> = sensors
            .iter()
            .flat_map(|s| {
                [
                    format!(
                        "SENSOR {} {} {} {} {} {}",
                        s.name,
                        s.kind,
                        s.update_rate,
                        u8::from(s.always_on),
                        u8::from(s.visualize),
                        s.topic
                    ),
                    format!(
                        "CUSTOM_TYPE {}",
                        s.custom()
                            .unwrap()
                            .custom_type
                            .as_deref()
                            .unwrap_or("ABSENT")
                    ),
                ]
            })
            .collect();
        assert_eq!(actual, expected, "native case {name}");
        for s in sensors {
            assert!(xml.contains(&s.custom().unwrap().xml));
            for extension in &s.extensions {
                assert!(xml.contains(&extension.xml));
                assert_eq!(extension.source.uri, "matrix.sdf");
            }
        }
    }
}

#[test]
fn large_custom_field_set_retains_source_order_and_duplicates() {
    // Thousands of distinct keys exercise dynamic membership. Repeated keys at
    // distant positions must remain separate values, not set-shaped output.
    const COUNT: usize = 4096;
    let mut xml = String::from("<sensor name='s' type='custom' gz:type='large'>\n");
    let mut expected = Vec::new();
    for i in 0..COUNT {
        let tag = format!("field_{i}");
        xml.push_str(&format!("<{tag}>{i}</{tag}>\n"));
        expected.push((tag, i.to_string()));
        if i % 128 == 0 {
            xml.push_str(&format!("<repeated>{i}</repeated>\n"));
            expected.push(("repeated".into(), i.to_string()));
        }
    }
    xml.push_str("</sensor>");
    let sensor = sensor(&xml).unwrap();
    assert_eq!(sensor.extensions.len(), expected.len());
    for (i, (item, (tag, value))) in sensor.extensions.iter().zip(expected).enumerate() {
        assert_eq!(item.tag, tag);
        assert_eq!(item.xml, format!("<{tag}>{value}</{tag}>"));
        assert_eq!(item.source.line as usize, i + 2);
        assert_eq!(item.source.column, 1);
        assert_eq!(item.source.uri, "custom.sdf");
    }
    assert_eq!(sensor.custom().unwrap().xml, xml);
    assert_eq!(
        sensor.custom().unwrap().custom_type.as_deref(),
        Some("large")
    );
}

#[test]
fn custom_metadata_default_clone_and_debug_retain_owned_values() {
    let mut copy = sdformat::CustomSensor::default();
    assert_eq!(copy.custom_type, None);
    assert!(copy.xml.is_empty());
    assert!(copy.element_name.is_empty());
    assert!(copy.parameters.is_empty());
    assert!(format!("{copy:?}").contains("custom_type: None"));
    let xml = String::from(
        "<sensor name='s' type='custom' gz:type='owned/type'><vendor:config><value>42</value></vendor:config></sensor>",
    );
    let parsed = sensor(&xml).unwrap();
    copy.clone_from(parsed.custom().unwrap());
    drop(parsed);
    assert_eq!(copy.xml, xml);
    assert_eq!(copy.custom_type.as_deref(), Some("owned/type"));
    assert_eq!(copy.parameter_text("value"), Some("42"));
    let mut edited = copy.clone();
    edited.custom_type.as_mut().unwrap().push_str("/edited");
    edited.xml.clear();
    assert_eq!(copy.xml, xml);
    assert_eq!(copy.custom_type.as_deref(), Some("owned/type"));
    let debug = format!("{copy:?}");
    assert!(debug.contains("owned/type"));
    assert!(debug.contains(&xml));
    assert!(debug.contains("vendor:config"));
}
