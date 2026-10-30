// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[path = "support/heightmap_projection.rs"]
mod projection;
fn data() -> Value {
    serde_json::from_str(include_str!("heightmap_xml.json")).unwrap()
}
#[test]
fn native_heightmap_inputs_and_current_outputs() {
    let d = data();
    assert_eq!(d["rows"].as_array().unwrap().len(), 28);
    for row in d["rows"].as_array().unwrap() {
        let loaded = load_heightmap_str(row["xml"].as_str().unwrap(), "heightmap.xml");
        if let Some(code) = row.get("gap_code") {
            let expected = match code.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown gap"),
            };
            assert_eq!(loaded.unwrap_err().code, expected, "{}", row["id"]);
        } else if row["export"] == true {
            let value = loaded.unwrap_or_else(|e| panic!("{}: {e:?}", row["id"]));
            let expected = projection::expected(row["native"]["stdout"].as_str().unwrap());
            assert_eq!(projection::projection(&value), expected, "{}", row["id"]);
            projection::assert_geometry(
                &format!("<geometry>{}</geometry>", row["xml"].as_str().unwrap()),
                "heightmap.xml",
                row["native"]["stdout"].as_str().unwrap(),
            );
            let before = value.clone();
            let xml = value.to_xml().unwrap();
            assert_eq!(value, before);
            assert_eq!(
                projection::projection(&load_heightmap_str(&xml, "roundtrip.xml").unwrap()),
                expected,
                "{}",
                row["id"]
            );
            if let Some(path) = std::env::var_os("SDF_HEIGHTMAP_XML_EXPORT") {
                std::fs::create_dir_all(&path).unwrap();
                std::fs::write(
                    std::path::PathBuf::from(path)
                        .join(format!("{}.xml", row["id"].as_str().unwrap())),
                    xml,
                )
                .unwrap();
            }
        } else {
            assert!(loaded.is_err(), "{}", row["id"]);
        }
    }
}
#[test]
fn output_rejects_unrepresentable_fields() {
    let base = load_heightmap_str("<heightmap><texture/><blend/></heightmap>", "base.xml").unwrap();
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
        for slot in 0..3 {
            let mut v = base.clone();
            match slot {
                0 => v.uri = text.into(),
                1 => v.textures[0].diffuse = text.into(),
                _ => v.textures[0].normal = text.into(),
            };
            let expected = if text.contains("]]>") {
                ErrorCode::Unsupported
            } else {
                ErrorCode::InvalidValue
            };
            assert_eq!(v.to_xml().unwrap_err().code, expected, "{text:?}");
        }
    }
    for value in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(1),
        -f64::from_bits(1),
    ] {
        for slot in 0..3 {
            let mut v = base.clone();
            match slot {
                0 => v.textures[0].size = value,
                1 => v.blends[0].min_height = value,
                _ => v.blends[0].fade_distance = value,
            };
            assert_eq!(v.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for slot in 0..6 {
            let mut v = base.clone();
            if slot < 3 {
                v.size[slot] = value;
            } else {
                v.position[slot - 3] = value;
            }
            assert_eq!(v.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        }
    }
}
#[test]
fn reader_errors_retain_caller_origin() {
    for xml in [
        "<heightmap bad='x'/>",
        "<heightmap><texture><unknown/></texture></heightmap>",
        "<heightmap><blend><fade_dist/></blend></heightmap>",
    ] {
        let error = load_heightmap_str(xml, "caller.xml").unwrap_err();
        assert_eq!(error.source.uri, "caller.xml");
        assert!(error.source.line > 0);
        assert!(error.source.column > 0);
    }
    assert!(load_heightmap_str("<mesh/>", "caller.xml").is_err());
    assert!(load_heightmap_str("<heightmap>", "caller.xml").is_err());
}
