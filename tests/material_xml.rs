// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[path = "support/material_xml_projection.rs"]
mod projection;
#[path = "support/quiet.rs"]
mod quiet;
fn data() -> Value {
    serde_json::from_str(include_str!("material_xml.json")).unwrap()
}
#[test]
fn native_material_inputs_and_lossless_metadata_output() {
    let d = data();
    assert_eq!(d["rows"].as_array().unwrap().len(), 53);
    for row in d["rows"].as_array().unwrap() {
        let loaded = load_material_str(row["xml"].as_str().unwrap(), "material.xml");
        if let Some(code) = row.get("gap_code") {
            let code = match code.as_str().unwrap() {
                "InvalidValue" => ErrorCode::InvalidValue,
                "Unsupported" => ErrorCode::Unsupported,
                _ => panic!("unknown gap"),
            };
            assert_eq!(loaded.unwrap_err().code, code, "{}", row["id"]);
        } else if row["export"] == true {
            let value = loaded.unwrap_or_else(|e| panic!("{}:{e:?}", row["id"]));
            let expected = projection::expected(row["native"]["stdout"].as_str().unwrap());
            assert_eq!(projection::projection(&value), expected, "{}", row["id"]);
            let raw = projection::raw_types(&value);
            let sources = projection::sources(&value);
            let xml = value
                .to_xml()
                .unwrap_or_else(|e| panic!("{}:{e:?}", row["id"]));
            assert_eq!(projection::projection(&value), expected);
            assert_eq!(projection::raw_types(&value), raw);
            assert_eq!(projection::sources(&value), sources);
            let reloaded = load_material_str(&xml, "roundtrip.xml").unwrap();
            assert_eq!(projection::projection(&reloaded), expected, "{}", row["id"]);
            assert_eq!(projection::raw_types(&reloaded), raw, "{}", row["id"]);
            if let Some(path) = std::env::var_os("SDF_MATERIAL_XML_EXPORT") {
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
fn raw_optional_attributes_have_literal_input_and_reload_oracles() {
    for attr in [
        None,
        Some(""),
        Some("object"),
        Some(" object "),
        Some("invalid"),
        Some("a & <b> \" ' "),
    ] {
        let attribute = attr
            .map(|v| format!(" type=\"{}\"", quick_xml::escape::escape(v)))
            .unwrap_or_default();
        let value=load_material_str(&format!("<material><pbr><metal><normal_map{attribute}>map</normal_map></metal></pbr></material>"),"raw.xml").unwrap();
        assert_eq!(
            value
                .pbr
                .as_ref()
                .unwrap()
                .metal
                .as_ref()
                .unwrap()
                .normal_map_type
                .as_deref(),
            attr
        );
        let loaded = load_material_str(&value.to_xml().unwrap(), "output.xml").unwrap();
        assert_eq!(
            loaded
                .pbr
                .as_ref()
                .unwrap()
                .metal
                .as_ref()
                .unwrap()
                .normal_map_type
                .as_deref(),
            attr
        );
    }
}
#[test]
fn exact_native_constructor_states_are_retained_and_explicitly_rejected() {
    for (kind, fixture, key) in [
        (
            PbrWorkflowKind::Metal,
            include_str!("fixtures/material_metal_native.xml"),
            "--fixture-metal",
        ),
        (
            PbrWorkflowKind::Specular,
            include_str!("fixtures/material_specular_native.xml"),
            "--fixture-specular",
        ),
    ] {
        let mut value = load_material_str(fixture, "constructor.xml").unwrap();
        let p = value.pbr.as_mut().unwrap();
        let flow = if kind == PbrWorkflowKind::Metal {
            p.metal.as_mut().unwrap()
        } else {
            p.specular.as_mut().unwrap()
        };
        flow.environment_map = "env".into();
        flow.roughness_map = "rough".into();
        flow.metalness_map = "metalness".into();
        flow.specular_map = "gloss".into();
        flow.glossiness_map = "gloss".into();
        flow.roughness = 0.1;
        flow.metalness = 1.;
        flow.normal_map_type = None;
        let expected: Vec<Vec<String>> = data()["constructors"][key]["stdout"]
            .as_str()
            .unwrap()
            .lines()
            .take(5)
            .filter(|s| !s.starts_with("RAW_SCRIPT"))
            .map(|s| s.split_whitespace().map(str::to_owned).collect())
            .collect();
        let actual: Vec<_> = projection::projection(&value)
            .into_iter()
            .filter(|v| v[0] != "RAW_SCRIPT")
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
}
#[test]
fn bounded_writer_rejects_lossy_values_without_mutating_reader_policy() {
    let base=load_material_str("<material><script><uri>uri</uri><name>name</name></script><pbr><metal/><specular/></pbr></material>","base.xml").unwrap();
    for bad in [f32::NAN, f32::INFINITY, -0.1, 1.1, f32::from_bits(1)] {
        let mut v = base.clone();
        v.ambient[0] = bad;
        assert!(v.to_xml().is_err());
    }
    for bad in [
        f32::from_bits(1),
        f32::from_bits(0x7fc00001),
        f32::from_bits(0xffc00001),
    ] {
        let mut v = base.clone();
        v.render_order = bad;
        assert!(v.to_xml().is_err());
    }
    for bad in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        let mut v = base.clone();
        v.shininess = bad;
        assert!(v.to_xml().is_err());
    }
    for bad in [
        f64::from_bits(1),
        f64::from_bits(0x7ff8000000000001),
        f64::from_bits(0xfff8000000000001),
    ] {
        let mut v = base.clone();
        v.pbr.as_mut().unwrap().metal.as_mut().unwrap().roughness = bad;
        assert!(v.to_xml().is_err());
    }
    let mut v = base.clone();
    v.pbr.as_mut().unwrap().metal.as_mut().unwrap().kind = PbrWorkflowKind::Specular;
    assert_eq!(v.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    for slot in 0..4 {
        let mut v = base.clone();
        let f = v.pbr.as_mut().unwrap().metal.as_mut().unwrap();
        match slot {
            0 => f.specular_map = "s".into(),
            1 => f.glossiness_map = "g".into(),
            2 => f.glossiness = -0.,
            _ => f.glossiness = 1.,
        };
        assert_eq!(v.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
    for slot in 0..4 {
        let mut v = base.clone();
        let f = v.pbr.as_mut().unwrap().specular.as_mut().unwrap();
        match slot {
            0 => f.roughness_map = "s".into(),
            1 => f.metalness_map = "g".into(),
            2 => f.roughness = 1.,
            _ => f.metalness = 1.,
        };
        assert_eq!(v.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    }
    for bad in ["", "__default__", " x", "x\r", "a  b]]>"] {
        let mut v = base.clone();
        v.script.as_mut().unwrap().name = bad.into();
        assert!(v.to_xml().is_err());
    }
    let mut v = base.clone();
    v.script.as_mut().unwrap().uris.clear();
    assert!(v.to_xml().is_err());
    let mut v = base.clone();
    v.script.as_mut().unwrap().uris.push(String::new());
    assert!(v.to_xml().is_err());
    for bad in ["", "__default__"] {
        let mut v = base.clone();
        v.script.as_mut().unwrap().uris[0] = bad.into();
        assert!(v.to_xml().is_err());
    }
    for shader in [
        ShaderType::NormalMapObjectSpace,
        ShaderType::NormalMapTangentSpace,
    ] {
        let mut v = base.clone();
        v.shader = shader;
        assert!(v.to_xml().is_err());
    }
    let mut v = base.clone();
    v.normal_map = "__default__".into();
    assert!(v.to_xml().is_err());
    for bad in ["x\n", "x\t", "x\r", "\0", "\u{fffe}", "\u{ffff}"] {
        let mut v = base.clone();
        v.pbr
            .as_mut()
            .unwrap()
            .metal
            .as_mut()
            .unwrap()
            .normal_map_type = Some(bad.into());
        assert!(v.to_xml().is_err());
    }
}
#[test]
fn standalone_errors_retain_caller_origin() {
    for xml in [
        "<material bad='x'/>",
        "<material><script/></material>",
        "<material><shader/></material>",
    ] {
        let error = load_material_str(xml, "caller.xml").unwrap_err();
        assert_eq!(error.source.uri, "caller.xml");
        assert!(error.source.line > 0);
        assert!(error.source.column > 0);
    }
    assert!(load_material_str("<camera/>", "caller.xml").is_err());
    assert!(load_material_str("<material>", "caller.xml").is_err());
}

#[test]
fn loaded_material_output_is_quiet_without_claiming_constructor_equivalence() {
    quiet::assert_quiet(
        "loaded_material_output_is_quiet_without_claiming_constructor_equivalence",
        "SDF_MATERIAL_XML_QUIET_CHILD",
        || {
            for xml in [
                include_str!("fixtures/material_metal_native.xml"),
                include_str!("fixtures/material_specular_native.xml"),
            ] {
                let value = load_material_str(xml, "loaded.xml").unwrap();
                let expected = projection::projection(&value);
                let loaded = load_material_str(&value.to_xml().unwrap(), "emitted.xml").unwrap();
                assert_eq!(projection::projection(&loaded), expected);
            }
        },
    );
}
