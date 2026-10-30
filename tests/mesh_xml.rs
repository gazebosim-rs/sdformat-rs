// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[path = "support/mesh_projection.rs"]
mod mesh_projection;
fn data() -> Value {
    serde_json::from_str(include_str!("mesh_xml.json")).unwrap()
}
#[test]
fn native_mesh_inputs_and_current_outputs() {
    let d = data();
    assert_eq!(d["rows"].as_array().unwrap().len(), 28);
    for row in d["rows"].as_array().unwrap() {
        let result = load_mesh_str(row["xml"].as_str().unwrap(), "mesh.xml");
        if let Some(code) = row.get("gap_code") {
            let code = match code.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown gap"),
            };
            assert_eq!(result.unwrap_err().code, code, "{}", row["id"]);
        } else if row["export"] == true {
            let value = result.unwrap_or_else(|e| panic!("{}: {e:?}", row["id"]));
            let expected = mesh_projection::expected(row["native"]["stdout"].as_str().unwrap());
            assert_eq!(
                mesh_projection::projection(&value),
                expected,
                "{}",
                row["id"]
            );
            mesh_projection::assert_geometry(
                &format!("<geometry>{}</geometry>", row["xml"].as_str().unwrap()),
                "mesh.xml",
                row["native"]["stdout"].as_str().unwrap(),
            );
            let before = value.clone();
            let xml = value.to_xml().unwrap();
            assert_eq!(value, before);
            assert_eq!(
                mesh_projection::projection(&load_mesh_str(&xml, "out.xml").unwrap()),
                expected,
                "{}",
                row["id"]
            );
            if let Some(path) = std::env::var_os("SDF_MESH_XML_EXPORT") {
                std::fs::create_dir_all(&path).unwrap();
                std::fs::write(
                    std::path::PathBuf::from(path)
                        .join(format!("{}.xml", row["id"].as_str().unwrap())),
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
fn output_rejects_unrepresentable_metadata() {
    let base = load_mesh_str("<mesh><uri>a</uri></mesh>", "base.xml").unwrap();
    for text in [
        "",
        " x",
        "x ",
        "\tx",
        "x\n",
        "x\r",
        "x\0",
        "x\u{fffe}",
        "x\u{ffff}",
        "a  b]]>",
        "a\tb]]>",
    ] {
        let mut value = base.clone();
        value.uri = text.into();
        assert!(value.to_xml().is_err(), "{text:?}");
        value = base.clone();
        value.submesh = text.into();
        if !text.is_empty() {
            assert!(value.to_xml().is_err(), "{text:?}");
        }
    }
    for scale in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut value = base.clone();
        value.scale[1] = scale;
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    let mut value = base.clone();
    value.center_submesh = true;
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    value.center_submesh = false;
    value.submesh = "__default__".into();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
}
#[test]
fn reader_diagnostics_retain_caller_location() {
    for xml in [
        "<mesh><unknown/></mesh>",
        "<mesh unknown='x'/>",
        "<mesh><scale>1 2 3 4</scale></mesh>",
        "<mesh><uri>x</uri><uri>y</uri></mesh>",
    ] {
        let error = load_mesh_str(xml, "caller.xml").unwrap_err();
        assert_eq!(error.source.uri, "caller.xml");
        assert!(error.source.line > 0);
        assert!(error.source.column > 0);
    }
    assert!(load_mesh_str("<geometry/>", "caller.xml").is_err());
    assert!(load_mesh_str("<mesh>", "caller.xml").is_err());
}
