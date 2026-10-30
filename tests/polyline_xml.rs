// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[path = "support/polyline_projection.rs"]
mod projection;
fn data() -> Value {
    serde_json::from_str(include_str!("polyline_xml.json")).unwrap()
}
#[test]
fn native_polyline_inputs_and_ordered_outputs() {
    let d = data();
    assert_eq!(d["rows"].as_array().unwrap().len(), 19);
    for row in d["rows"].as_array().unwrap() {
        let xml = row["xml"].as_str().unwrap();
        let result = if xml.starts_with("<geometry") {
            load_geometry_str(xml, "polyline.xml").map(|g| {
                let Geometry::Polyline(lines) = g else {
                    panic!("not Polyline")
                };
                lines
            })
        } else {
            load_polyline_str(xml, "polyline.xml").map(|p| vec![p])
        };
        if let Some(code) = row.get("gap_code") {
            let code = match code.as_str().unwrap() {
                "InvalidValue" => ErrorCode::InvalidValue,
                "Unsupported" => ErrorCode::Unsupported,
                _ => panic!("unknown gap"),
            };
            assert_eq!(result.unwrap_err().code, code, "{}", row["id"]);
        } else if row["export"] == true {
            let lines = result.unwrap_or_else(|e| panic!("{}:{e:?}", row["id"]));
            let expected = projection::expected(row["native"]["stdout"].as_str().unwrap());
            assert_eq!(projection::projection(&lines), expected, "{}", row["id"]);
            let geometry_xml = if xml.starts_with("<geometry") {
                xml.to_owned()
            } else {
                format!("<geometry>{xml}</geometry>")
            };
            projection::assert_geometry(
                &geometry_xml,
                "polyline.xml",
                row["native"]["stdout"].as_str().unwrap(),
            );
            let value = Geometry::Polyline(lines);
            let before = value.clone();
            let output = value.to_xml().unwrap();
            assert_eq!(value, before);
            let Geometry::Polyline(loaded) = load_geometry_str(&output, "roundtrip.xml").unwrap()
            else {
                panic!("not Polyline")
            };
            assert_eq!(projection::projection(&loaded), expected, "{}", row["id"]);
            if !xml.starts_with("<geometry") {
                let Geometry::Polyline(lines) = value else {
                    unreachable!()
                };
                let single = lines[0].to_xml().unwrap();
                assert_eq!(
                    projection::projection(&[load_polyline_str(&single, "single.xml").unwrap()]),
                    expected
                );
            }
            if let Some(path) = std::env::var_os("SDF_POLYLINE_XML_EXPORT") {
                std::fs::create_dir_all(&path).unwrap();
                std::fs::write(
                    std::path::PathBuf::from(path)
                        .join(format!("{}.xml", row["id"].as_str().unwrap())),
                    output,
                )
                .unwrap();
            }
        } else {
            assert!(result.is_err(), "{}", row["id"]);
        }
    }
}
#[test]
fn output_rejects_lost_points_and_numeric_states() {
    let base = load_polyline_str("<polyline/>", "base.xml").unwrap();
    let mut empty = base.clone();
    empty.points.clear();
    assert_eq!(empty.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    assert_eq!(
        Geometry::Polyline(vec![]).to_xml().unwrap_err().code,
        ErrorCode::Unsupported
    );
    for height in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(1),
        -f64::from_bits(1),
    ] {
        let mut line = base.clone();
        line.height = height;
        assert_eq!(line.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    for point in [[f64::NAN, 0.], [0., f64::INFINITY], [f64::NEG_INFINITY, 0.]] {
        let mut line = base.clone();
        line.points[0] = point;
        assert_eq!(line.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
}
#[test]
fn reader_errors_retain_caller_origin() {
    for xml in [
        "<polyline bad='x'/>",
        "<polyline><point>1</point></polyline>",
        "<polyline><height/></polyline>",
    ] {
        let error = load_polyline_str(xml, "caller.xml").unwrap_err();
        assert_eq!(error.source.uri, "caller.xml");
        assert!(error.source.line > 0);
        assert!(error.source.column > 0);
    }
    assert!(load_polyline_str("<mesh/>", "caller.xml").is_err());
    assert!(load_polyline_str("<polyline>", "caller.xml").is_err());
}
