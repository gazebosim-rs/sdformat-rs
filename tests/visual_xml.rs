/*
 * Copyright (C) 2018 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#![cfg(all(target_os = "linux", target_env = "gnu"))]
use sdformat::*;
fn same(a: &Visual, b: &Visual) {
    assert_eq!(a.name, b.name);
    assert_eq!(a.pose.relative_to, b.pose.relative_to);
    for (x, y) in a
        .pose
        .transform
        .translation
        .iter()
        .chain(&a.pose.transform.rotation)
        .zip(
            b.pose
                .transform
                .translation
                .iter()
                .chain(&b.pose.transform.rotation),
        )
    {
        assert!((x - y).abs() <= 1e-3);
    }
    assert_eq!(a.geometry, b.geometry);
    assert_eq!(a.properties, b.properties);
    assert_eq!(a.material, b.material);
    assert_eq!(a.plugins.len(), b.plugins.len());
    for (x, y) in a.plugins.iter().zip(&b.plugins) {
        assert_eq!(x.name, y.name);
        assert_eq!(x.filename, y.filename);
        assert_eq!(x.xml, y.xml);
    }
}
#[test]
fn configured_empty_and_shape_metadata_roundtrip() {
    for geometry in [
        "",
        "<geometry/>",
        "<geometry><box><size>2 3 4</size></box></geometry>",
    ] {
        let xml = format!(
            "<visual name='v'><pose relative_to='f'>1 2 3 .1 .2 .3</pose>{geometry}<transparency>.2</transparency><cast_shadows>false</cast_shadows><visibility_flags>1234</visibility_flags><laser_retro>1.2</laser_retro><material/><plugin name='p' filename='f'><custom a='1'/></plugin></visual>"
        );
        let value = load_visual_str(&xml, "input.xml").unwrap();
        let output = value.to_xml().unwrap();
        let mut loaded = load_visual_str(&output, "output.xml").unwrap();
        loaded.material.as_mut().unwrap().source = value.material.as_ref().unwrap().source.clone();
        same(&value, &loaded);
        assert_eq!(value.source.uri, "input.xml");
        assert_eq!(loaded.source.uri, "output.xml");
    }
}
#[test]
fn scalar_bits_and_retro_presence_are_preserved() {
    let mut value = load_visual_str("<visual name='v'/>", "x").unwrap();
    for n in [0., -0., 0.2, -1., f32::MAX, f32::MIN_POSITIVE] {
        value.properties.transparency = n;
        for retro in [None, Some(0.), Some(-0.), Some(-1.)] {
            value.properties.laser_retro = retro;
            let loaded = load_visual_str(&value.to_xml().unwrap(), "y").unwrap();
            assert_eq!(n.to_bits(), loaded.properties.transparency.to_bits());
            assert_eq!(
                retro.map(f64::to_bits),
                loaded.properties.laser_retro.map(f64::to_bits)
            );
        }
    }
    for n in [f32::NAN, f32::INFINITY, f32::from_bits(1)] {
        value.properties.transparency = n;
        assert!(value.to_xml().is_err());
    }
    value.properties.transparency = 0.;
    for n in [f64::NAN, f64::INFINITY, f64::from_bits(1)] {
        value.properties.laser_retro = Some(n);
        assert!(value.to_xml().is_err());
    }
}
#[test]
fn raw_plugin_payload_cannot_disagree_with_owned_attributes() {
    let mut value = load_visual_str(
        "<visual name='v'><plugin name='p' filename='f'><x:extension x:a='b'/></plugin></visual>",
        "x",
    )
    .unwrap();
    assert!(value.to_xml().unwrap().contains("<x:extension x:a='b'/>"));
    value.plugins[0].filename = "other".into();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Unsupported);
    value.plugins[0].xml.clear();
    let loaded = load_visual_str(&value.to_xml().unwrap(), "y").unwrap();
    assert_eq!(loaded.plugins[0].filename, "other");
    value.plugins[0].xml = "<broken".into();
    assert_eq!(value.to_xml().unwrap_err().code, ErrorCode::Xml);
    value.plugins[0].xml.clear();
    value.plugins[0].filename.clear();
    assert!(value.to_xml().is_err());
}
#[test]
fn standalone_errors_retain_caller_location_and_strict_document_boundaries() {
    for xml in [
        "<visual/>",
        "<visual name='v'><unknown/></visual>",
        "<visual name='v'><geometry><empty/></geometry></visual>",
        "<visual name='v'><geometry><box/></geometry><geometry/></visual>",
    ] {
        let error = load_visual_str(xml, "caller.xml").unwrap_err();
        assert_eq!(error.source.uri, "caller.xml");
        assert_eq!(error.source.line, 1);
        assert!(error.source.column > 0);
    }
    assert_eq!(
        load_visual_str("<material/>", "x").unwrap_err().code,
        ErrorCode::Xml
    );
    assert_eq!(
        load_visual_str("<visual", "x").unwrap_err().code,
        ErrorCode::Xml
    );
    let mut value = load_visual_str("<visual name='v'/>", "x").unwrap();
    for name in ["", "world", "__bad__", "a::b", " bad", "bad\n"] {
        value.name = name.into();
        assert!(value.to_xml().is_err());
    }
    value.name = "v".into();
    value.pose.transform.translation[0] = f64::from_bits(1);
    assert!(value.to_xml().is_err());
    value.pose.transform.translation[0] = f64::INFINITY;
    assert!(value.to_xml().is_err());
}

#[test]
fn original_visual_to_element() {
    // Visual_TEST.cc DOMVisual.ToElement: native stores EMPTY Geometry by value.
    let mut visual = load_visual_str("<visual name='my-visual'/>", "setup.xml").unwrap();
    visual.properties.cast_shadows = true;
    visual.properties.transparency = 0.2;
    visual.pose.transform.translation = [1., 2., 3.];
    visual.pose.transform.rotation =
        glam::DQuat::from_euler(glam::EulerRot::ZYX, 0.3, 0.2, 0.1).to_array();
    visual.properties.visibility_flags = 1234;
    visual.properties.laser_retro = Some(1.2);
    visual.geometry = None;
    visual.material = Some(Box::new(
        load_material_str("<material/>", "setup-material.xml").unwrap(),
    ));
    visual.plugins.push(Plugin {
        name: "name1".into(),
        filename: "filename1".into(),
        xml: String::new(),
    });
    let xml = visual.to_xml().unwrap();
    assert!(!xml.is_empty());
    let loaded = load_visual_str(&xml, "loaded.xml").unwrap();
    assert_eq!(visual.name, loaded.name);
    assert_eq!(
        visual.properties.cast_shadows,
        loaded.properties.cast_shadows
    );
    assert_eq!(
        visual.properties.transparency,
        loaded.properties.transparency
    );
    for (a, b) in visual
        .pose
        .transform
        .translation
        .iter()
        .chain(&visual.pose.transform.rotation)
        .zip(
            loaded
                .pose
                .transform
                .translation
                .iter()
                .chain(&loaded.pose.transform.rotation),
        )
    {
        assert!((a - b).abs() <= 1e-3);
    }
    assert_eq!(
        visual.properties.visibility_flags,
        loaded.properties.visibility_flags
    );
    assert_eq!(
        visual.properties.laser_retro.is_some(),
        loaded.properties.laser_retro.is_some()
    );
    assert_eq!(visual.properties.laser_retro, loaded.properties.laser_retro);
    // Native Geom() always addresses its owned Geometry; None denotes its EMPTY type.
    assert!(loaded.geometry.is_none());
    assert!(loaded.material.is_some());
    assert_eq!(1, loaded.plugins.len());
    assert_eq!("name1", loaded.plugins[0].name);
    assert_eq!("filename1", loaded.plugins[0].filename);
}

fn native_projection(value: &Visual) -> Vec<String> {
    let mut fields = vec![value.name.clone(), value.pose.relative_to.clone()];
    fields.extend(
        value
            .pose
            .transform
            .translation
            .iter()
            .chain(&value.pose.transform.rotation)
            .map(|v| v.to_bits().to_string()),
    );
    fields.push(u8::from(value.properties.cast_shadows).to_string());
    fields.push(
        f64::from(value.properties.transparency)
            .to_bits()
            .to_string(),
    );
    fields.push(value.properties.visibility_flags.to_string());
    fields.push(u8::from(value.properties.laser_retro.is_some()).to_string());
    fields.push(
        value
            .properties
            .laser_retro
            .unwrap_or(0.)
            .to_bits()
            .to_string(),
    );
    fields.push("1".into());
    let kind = match &value.geometry {
        None => 0,
        Some(Geometry::Box { .. }) => 1,
        Some(Geometry::Cylinder { .. }) => 2,
        Some(Geometry::Plane { .. }) => 3,
        Some(Geometry::Sphere { .. }) => 4,
        Some(Geometry::Mesh(_)) => 5,
        Some(Geometry::Heightmap(_)) => 6,
        Some(Geometry::Polyline(_)) => 9,
        Some(Geometry::Capsule { .. }) => 7,
        Some(Geometry::Ellipsoid { .. }) => 8,
        Some(Geometry::Cone { .. }) => 10,
    };
    fields.push(kind.to_string());
    fields.push(u8::from(value.material.is_some()).to_string());
    fields.push(value.plugins.len().to_string());
    for plugin in &value.plugins {
        fields.push(plugin.name.clone());
        fields.push(plugin.filename.clone());
    }
    fields
}
fn compare_native(value: &Visual, expected: &[String]) {
    let actual = native_projection(value);
    assert_eq!(actual.len(), expected.len());
    for (i, (a, b)) in actual.iter().zip(expected).enumerate() {
        if (2..9).contains(&i) {
            assert!(
                (f64::from_bits(a.parse().unwrap()) - f64::from_bits(b.parse().unwrap())).abs()
                    <= 1e-3,
                "pose coordinate {i}"
            );
        } else {
            assert_eq!(a, b, "field {i}");
        }
    }
}
#[test]
fn actual_native_corpus_and_output_exchange() {
    let data: serde_json::Value = serde_json::from_str(include_str!("visual_xml.json")).unwrap();
    assert_eq!(28, data["rows"].as_array().unwrap().len());
    for row in data["rows"].as_array().unwrap() {
        let loaded = load_visual_str(row["xml"].as_str().unwrap(), "visual.xml");
        if let Some(gap) = row.get("gap") {
            let code = match gap.as_str().unwrap() {
                "Missing" => ErrorCode::Missing,
                "Unsupported" => ErrorCode::Unsupported,
                _ => panic!("unknown gap"),
            };
            assert_eq!(loaded.unwrap_err().code, code, "{}", row["id"]);
            continue;
        }
        let value = loaded.unwrap_or_else(|e| panic!("{}:{e:?}", row["id"]));
        let expected: Vec<String> = serde_json::from_value(row["values"].clone()).unwrap();
        compare_native(&value, &expected);
        let output = value.to_xml().unwrap();
        let reloaded = load_visual_str(&output, "roundtrip.xml").unwrap();
        compare_native(&reloaded, &expected);
        if let Some(path) = std::env::var_os("SDF_VISUAL_XML_EXPORT") {
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(
                std::path::PathBuf::from(path).join(format!("{}.xml", row["id"].as_str().unwrap())),
                output,
            )
            .unwrap();
        }
    }
}
