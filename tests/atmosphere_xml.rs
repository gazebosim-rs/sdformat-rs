// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
#[test]
fn typed_atmosphere_values_and_immutable_input_origin() {
    for n in [0., -0., -1., 123., f64::MAX, f64::MIN_POSITIVE] {
        let value = Atmosphere {
            temperature_kelvin: n,
            temperature_gradient: n,
            pressure: n,
            ..Atmosphere::default()
        };
        let xml = value.to_xml().unwrap();
        let loaded = load_atmosphere_str(&xml, "input.xml").unwrap();
        for v in [
            loaded.temperature_kelvin,
            loaded.temperature_gradient,
            loaded.pressure,
        ] {
            assert_eq!(n.to_bits(), v.to_bits());
        }
        assert!(value.source.is_none());
        assert!(value.xml.is_none());
        assert_eq!(
            loaded.source.as_ref().unwrap(),
            &Source {
                uri: "input.xml".into(),
                line: 1,
                column: 1
            }
        );
        assert_eq!(loaded.xml.as_deref(), Some(xml.as_str()));
        let mut edited = loaded;
        edited.pressure = 42.;
        let before = edited.xml.clone();
        let origin = edited.source.clone();
        let output = edited.to_xml().unwrap();
        assert_eq!(edited.xml, before);
        assert_eq!(edited.source, origin);
        assert_eq!(
            42.,
            load_atmosphere_str(&output, "edited.xml").unwrap().pressure
        );
    }
}
#[test]
fn explicit_type_numeric_and_unknown_boundaries_are_unchanged() {
    for xml in ["<atmosphere/>", "<atmosphere type=''/>"] {
        assert_eq!(
            load_atmosphere_str(xml, "x").unwrap_err().code,
            ErrorCode::Missing
        );
    }
    for xml in [
        "<atmosphere type='other'/>",
        "<atmosphere type='adiabatic'><temperature>nan</temperature></atmosphere>",
    ] {
        assert_eq!(
            load_atmosphere_str(xml, "x").unwrap_err().code,
            ErrorCode::InvalidValue
        );
    }
    for xml in [
        "<atmosphere type='adiabatic' extra='x'/>",
        "<atmosphere type='adiabatic'><unknown/></atmosphere>",
    ] {
        assert_eq!(
            load_atmosphere_str(xml, "x").unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
    assert_eq!(load_atmosphere_str("<atmosphere type='adiabatic'><pressure>1</pressure><pressure>2</pressure></atmosphere>", "x").unwrap_err().code, ErrorCode::InvalidValue);
    for xml in ["<atmosphere", "<material/>"] {
        assert_eq!(
            load_atmosphere_str(xml, "caller.xml").unwrap_err().code,
            ErrorCode::Xml
        );
    }
    for n in [
        f64::INFINITY,
        f64::NAN,
        f64::from_bits(1),
        -f64::from_bits(1),
    ] {
        for i in 0..3 {
            let mut value = Atmosphere::default();
            match i {
                0 => value.temperature_kelvin = n,
                1 => value.temperature_gradient = n,
                _ => value.pressure = n,
            };
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
}

fn projection(value: &Atmosphere) -> Vec<String> {
    let kind = match value.atmosphere_type {
        AtmosphereType::Adiabatic => 0,
    };
    vec![
        kind.to_string(),
        value.temperature_kelvin.to_bits().to_string(),
        value.temperature_gradient.to_bits().to_string(),
        value.pressure.to_bits().to_string(),
    ]
}
#[test]
fn actual_native_atmosphere_inputs_and_output_exchange() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("atmosphere_xml.json")).unwrap();
    assert_eq!(23, data["rows"].as_array().unwrap().len());
    for row in data["rows"].as_array().unwrap() {
        let result = load_atmosphere_str(row["xml"].as_str().unwrap(), "atmosphere.xml");
        if let Some(gap) = row.get("gap") {
            let code = match gap.as_str().unwrap() {
                "InvalidValue" => ErrorCode::InvalidValue,
                "Missing" => ErrorCode::Missing,
                "Unsupported" => ErrorCode::Unsupported,
                _ => panic!("unknown gap"),
            };
            assert_eq!(result.unwrap_err().code, code, "{}", row["id"]);
            continue;
        }
        let value = result.unwrap();
        let expected: Vec<String> = serde_json::from_value(row["values"].clone()).unwrap();
        assert_eq!(projection(&value), expected, "{}", row["id"]);
        let origin = value.source.clone();
        let raw = value.xml.clone();
        let output = value.to_xml().unwrap();
        assert_eq!(value.source, origin);
        assert_eq!(value.xml, raw);
        assert_eq!(
            projection(&load_atmosphere_str(&output, "reload.xml").unwrap()),
            expected
        );
        if let Some(path) = std::env::var_os("SDF_ATMOSPHERE_XML_EXPORT") {
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(
                std::path::PathBuf::from(path).join(format!("{}.xml", row["id"].as_str().unwrap())),
                output,
            )
            .unwrap();
        }
    }
    let native_fixture = include_str!("fixtures/atmosphere_to_element_native.xml");
    let configured = load_atmosphere_str(native_fixture, "native-fixture.xml").unwrap();
    assert_eq!(configured.temperature_kelvin, 123.);
    assert_eq!(configured.temperature_gradient, 1.34);
    assert_eq!(configured.pressure, 2.65);
}
