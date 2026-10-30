// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
fn empty(value: &Geometry) {
    assert_eq!(value, &Geometry::Empty);
    let xml = value.to_xml().unwrap();
    assert_eq!(xml, "<geometry><empty/></geometry>");
    assert_eq!(
        load_geometry_str(&xml, "reload.xml").unwrap(),
        Geometry::Empty
    );
}
#[test]
fn default_and_empty_groups_roundtrip() {
    empty(&Geometry::default());
    for xml in [
        "<geometry><empty/></geometry>",
        "<geometry><!--keep--><empty/></geometry>",
        "<geometry>ignored text</geometry>",
        "<geometry><empty>ignored text</empty></geometry>",
    ] {
        empty(&load_geometry_str(xml, "empty.xml").unwrap());
    }
}
#[test]
fn visual_absent_and_bare_groups_keep_their_existing_none_representation() {
    for xml in [
        "<visual name='v'/>",
        "<visual name='v'><geometry/></visual>",
    ] {
        let value = load_visual_str(xml, "legacy.xml").unwrap();
        assert!(value.geometry.is_none());
        assert!(
            load_visual_str(&value.to_xml().unwrap(), "again.xml")
                .unwrap()
                .geometry
                .is_none()
        );
    }
}
#[test]
fn strict_structure_remains_an_explicit_native_difference() {
    for (xml, code) in [
        (
            "<geometry bad='x'><empty/></geometry>",
            ErrorCode::Unsupported,
        ),
        (
            "<geometry><empty bad='x'/></geometry>",
            ErrorCode::Unsupported,
        ),
        (
            "<geometry><empty><box/></empty></geometry>",
            ErrorCode::Unsupported,
        ),
        ("<geometry><unknown/></geometry>", ErrorCode::Unsupported),
        (
            "<geometry><empty/><empty/></geometry>",
            ErrorCode::InvalidValue,
        ),
        (
            "<geometry><empty/><box/></geometry>",
            ErrorCode::InvalidValue,
        ),
        (
            "<geometry><box/><empty/></geometry>",
            ErrorCode::InvalidValue,
        ),
    ] {
        let error = load_geometry_str(xml, "strict.xml").unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(error.source.uri, "strict.xml");
        assert_eq!(error.source.line, 1);
    }
}
#[test]
fn native_empty_fixture_and_all_metadata_contexts_match() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("geometry_empty.json")).unwrap();
    assert_eq!(38, data["rows"].as_array().unwrap().len());
    assert_eq!(
        load_geometry_str(
            include_str!("fixtures/geometry_empty_native.xml"),
            "native.xml"
        )
        .unwrap(),
        Geometry::Empty
    );
    for row in data["rows"].as_array().unwrap() {
        let xml = row["xml"].as_str().unwrap();
        let mode = row["mode"].as_str().unwrap();
        let parsed = match mode {
            "geometry" => {
                load_geometry_str(xml, "matrix.xml").map(|v| (v.to_xml().unwrap(), "geometry"))
            }
            "visual" => load_visual_str(xml, "matrix.xml").map(|v| {
                let absent = row["visual_none"].as_bool().unwrap();
                assert_eq!(
                    v.geometry,
                    if absent { None } else { Some(Geometry::Empty) }
                );
                (v.to_xml().unwrap(), "visual")
            }),
            "collision" | "document" => {
                let document = if mode == "document" {
                    xml.into()
                } else {
                    format!(
                        "<sdf version='1.12'><model name='m'><link name='l'>{xml}</link></model></sdf>"
                    )
                };
                load_str(&document, "matrix.xml", &NoResources).map(|v| {
                    let g = &v.models[0].links[0].collisions[0].geometry;
                    assert_eq!(g, &Geometry::Empty);
                    (g.to_xml().unwrap(), "geometry")
                })
            }
            _ => panic!("unknown mode"),
        };
        if let Some(code) = row.get("rust_error") {
            let code = match code.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown code"),
            };
            assert_eq!(parsed.unwrap_err().code, code, "{}", row["id"]);
            continue;
        }
        let (output, export_mode) = parsed.unwrap_or_else(|e| panic!("{}:{e:?}", row["id"]));
        if let Some(path) = std::env::var_os("SDF_EMPTY_GEOMETRY_EXPORT") {
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(
                std::path::PathBuf::from(&path)
                    .join(format!("{}.xml", row["id"].as_str().unwrap())),
                output,
            )
            .unwrap();
            std::fs::write(
                std::path::PathBuf::from(path)
                    .join(format!("{}.mode", row["id"].as_str().unwrap())),
                export_mode,
            )
            .unwrap();
        }
    }
}
