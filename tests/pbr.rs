// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;

fn hex(value: &str) -> String {
    if value.is_empty() {
        return "-".into();
    }
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn projection(pbr: Option<&PbrMaterial>) -> Vec<String> {
    let mut tokens = vec!["0".into(), u8::from(pbr.is_some()).to_string()];
    if let Some(pbr) = pbr {
        for workflow in [&pbr.metal, &pbr.specular] {
            tokens.push(u8::from(workflow.is_some()).to_string());
            let Some(flow) = workflow else {
                continue;
            };
            tokens.push(
                match flow.kind {
                    PbrWorkflowKind::Metal => "1",
                    PbrWorkflowKind::Specular => "2",
                }
                .into(),
            );
            for map in [
                &flow.albedo_map,
                &flow.normal_map,
                &flow.environment_map,
                &flow.ambient_occlusion_map,
                &flow.emissive_map,
                &flow.light_map,
                &flow.roughness_map,
                &flow.metalness_map,
                &flow.specular_map,
                &flow.glossiness_map,
            ] {
                tokens.push(hex(map));
            }
            for value in [flow.roughness, flow.metalness, flow.glossiness] {
                tokens.push(value.to_bits().to_string());
            }
            tokens.push(flow.light_map_uv_set.to_string());
        }
    }
    tokens
}
#[test]
fn native_workflow_values_and_admission_match() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("pbr.json")).unwrap();
    for row in rows {
        let expected: Vec<String> = row["tokens"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        let actual = load_str(row["xml"].as_str().unwrap(), "pbr.sdf", &NoResources);
        assert_eq!(
            actual.is_ok(),
            expected[0] == "0",
            "{}: {actual:?}",
            row["id"]
        );
        if let Ok(sdf) = actual {
            let material = sdf.models[0].links[0].visuals[0].material.as_ref().unwrap();
            assert_eq!(
                projection(material.pbr.as_deref()),
                expected,
                "{}",
                row["id"]
            );
        }
    }
}

#[test]
fn maps_keep_declared_attributes_and_owned_source_after_clone_and_drop() {
    let saved = {
        let sdf = load_str(
            include_str!("fixtures/material_pbr.sdf"),
            "resource://material.sdf",
            &NoResources,
        )
        .unwrap();
        let material = sdf.models[0].links[0].visuals[2].material.as_ref().unwrap();
        material.pbr.clone().unwrap()
    };
    let mut edited = saved.clone();
    edited.metal.as_mut().unwrap().albedo_map = "edited.png".into();
    let metal = saved.metal.as_ref().unwrap();
    let specular = saved.specular.as_ref().unwrap();
    assert_eq!(metal.albedo_map, "albedo_map.png");
    assert_eq!(metal.normal_map_type.as_deref(), Some("object"));
    assert_eq!(specular.normal_map_type.as_deref(), Some("tangent"));
    assert_eq!(saved.source.uri, "resource://material.sdf");
    assert_eq!(metal.source.uri, saved.source.uri);
    assert!(metal.source.line > saved.source.line);
}
#[test]
fn unsupported_fields_and_malformed_shapes_remain_explicit_errors() {
    for body in [
        "<pbr bad='x'/>",
        "<pbr><metal bad='x'/></pbr>",
        "<pbr><metal><albedo_map bad='x'>a</albedo_map></metal></pbr>",
        "<pbr><metal><normal_map bad='x'>a</normal_map></metal></pbr>",
        "<pbr><metal><light_map bad='x'>a</light_map></metal></pbr>",
        "<pbr><metal><normal_map><nested/></normal_map></metal></pbr>",
        "<pbr><metal><roughness><nested/></roughness></metal></pbr>",
        "<pbr><metal><specular_map>a</specular_map></metal></pbr>",
        "<pbr><specular><roughness>0.2</roughness></specular></pbr>",
        "<pbr><metal><albedo_map>a</albedo_map><albedo_map>b</albedo_map></metal></pbr>",
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><visual name='v'><material>{body}</material></visual></link></model></sdf>"
        );
        assert!(load_str(&xml, "bad.sdf", &NoResources).is_err(), "{body}");
    }
}
