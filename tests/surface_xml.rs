// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[path = "support/surface_projection.rs"]
mod projection;
fn configured() -> Surface {
    load_surface_str(
        include_str!("fixtures/surface_to_element_native.xml"),
        "configured.xml",
    )
    .unwrap()
}
#[test]
fn native_inputs_and_actual_outputs_keep_all_supported_fields() {
    let data: Vec<Value> = serde_json::from_str(include_str!("surface_xml.json")).unwrap();
    assert_eq!(data.len(), 34);
    let export = std::env::var_os("SDF_SURFACE_XML_EXPORT").map(std::path::PathBuf::from);
    if let Some(path) = &export {
        std::fs::create_dir_all(path).unwrap();
    }
    for row in data {
        let result = load_surface_str(row["xml"].as_str().unwrap(), "surface.xml");
        if let Some(gap) = row.get("gap_code") {
            let code = match gap.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown boundary"),
            };
            assert_eq!(
                result.expect_err(row["id"].as_str().unwrap()).code,
                code,
                "{}",
                row["id"]
            );
        } else if row["export"] == true {
            let value = result.unwrap_or_else(|e| panic!("{}:{e:?}", row["id"]));
            let native: Vec<_> = row["native"]["stdout"]
                .as_str()
                .unwrap()
                .split_whitespace()
                .map(str::to_owned)
                .collect();
            assert_eq!(projection::projection(&value), native, "{}", row["id"]);
            let xml = value.to_xml().unwrap();
            assert_eq!(
                projection::projection(&load_surface_str(&xml, "roundtrip.xml").unwrap()),
                native
            );
            if let Some(path) = &export {
                std::fs::write(
                    path.join(format!("{}.xml", row["id"].as_str().unwrap())),
                    xml,
                )
                .unwrap();
            }
        } else {
            assert!(result.is_err(), "{}", row["id"]);
        }
    }
}
#[test]
fn output_rejects_unrepresentable_scalar_and_vector_states() {
    for field in 0..11 {
        for bad in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::from_bits(1),
        ] {
            let mut value = configured();
            match field {
                0 => value.friction.ode.mu = bad,
                1 => value.friction.ode.mu2 = bad,
                2 => value.friction.ode.slip1 = bad,
                3 => value.friction.ode.slip2 = bad,
                4 => value.friction.bullet.as_mut().unwrap().friction = bad,
                5 => value.friction.bullet.as_mut().unwrap().friction2 = bad,
                6 => value.friction.bullet.as_mut().unwrap().rolling_friction = bad,
                7 => value.friction.torsional.as_mut().unwrap().coefficient = bad,
                8 => value.friction.torsional.as_mut().unwrap().patch_radius = bad,
                9 => value.friction.torsional.as_mut().unwrap().surface_radius = bad,
                _ => value.friction.torsional.as_mut().unwrap().ode_slip = bad,
            }
            assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    for field in 0..7 {
        let mut value = configured();
        match field {
            0 => value.friction.ode.mu = -1.,
            1 => value.friction.ode.mu2 = -1.,
            2 => value.friction.bullet.as_mut().unwrap().friction = -1.,
            3 => value.friction.bullet.as_mut().unwrap().friction2 = -1.,
            4 => value.friction.torsional.as_mut().unwrap().coefficient = -1.,
            5 => value.friction.torsional.as_mut().unwrap().patch_radius = -1.,
            _ => value.friction.torsional.as_mut().unwrap().surface_radius = -1.,
        }
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    for bullet in [false, true] {
        for coordinate in 0..3 {
            for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                let mut value = configured();
                if bullet {
                    value.friction.bullet.as_mut().unwrap().fdir1[coordinate] = bad;
                } else {
                    value.friction.ode.fdir1[coordinate] = bad;
                }
                assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
            }
        }
    }
}
#[test]
fn root_errors_and_nested_diagnostics_retain_the_caller_source() {
    assert!(load_surface_str("<", "bad.xml").is_err());
    assert_eq!(
        load_surface_str("<frame/>", "bad.xml").unwrap_err().code,
        ErrorCode::Xml
    );
    let error = load_surface_str(
        "<surface>\n<friction><ode><mu>-1</mu></ode></friction></surface>",
        "caller.xml",
    )
    .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!(
        error.source,
        Source {
            uri: "caller.xml".into(),
            line: 2,
            column: 16
        }
    );
    for xml in [
        "<surface bad='x'/>",
        "<surface><friction><ode><mu bad='x'>1</mu></ode></friction></surface>",
        "<surface><contact><unknown/></contact></surface>",
    ] {
        assert_eq!(
            load_surface_str(xml, "caller.xml").unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
}
#[test]
fn emission_keeps_actual_input_origins_and_default_values_owned() {
    let value = Surface::default();
    assert!(value.source.is_none());
    let xml = value.to_xml().unwrap();
    assert!(value.source.is_none());
    let parsed = load_surface_str(&xml, "surface-origin.xml").unwrap();
    assert_eq!(
        parsed.source,
        Some(Source {
            uri: "surface-origin.xml".into(),
            line: 1,
            column: 1
        })
    );
    assert_eq!(
        projection::projection(&parsed),
        projection::projection(&value)
    );
    let before = parsed.clone();
    parsed.to_xml().unwrap();
    assert_eq!(parsed, before);
    assert!(parsed.contact.source.is_some());
    assert!(parsed.friction.source.is_some());
    assert!(parsed.friction.ode.source.is_some());
}
