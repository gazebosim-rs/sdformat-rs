// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{NoResources, ShaderType, load_str};
use serde_json::Value;
fn text(token: &Value) -> String {
    let s = token.as_str().unwrap();
    if s == "-" {
        return String::new();
    }
    let bytes: Vec<_> = s
        .as_bytes()
        .chunks_exact(2)
        .map(|c| u8::from_str_radix(std::str::from_utf8(c).unwrap(), 16).unwrap())
        .collect();
    String::from_utf8(bytes).unwrap()
}
#[test]
fn native_material_values_and_errors() {
    let cases: Vec<Value> = serde_json::from_str(include_str!("material.json")).unwrap();
    for row in cases {
        let result = load_str(row["xml"].as_str().unwrap(), "material.sdf", &NoResources);
        let tokens = row["tokens"].as_array().unwrap();
        let accepted = tokens[0] == "0";
        if row.get("gap").is_some() {
            assert!(accepted);
            let code = result.unwrap_err().code;
            assert_eq!(
                format!("{code:?}"),
                row["expected_error"].as_str().unwrap_or("Unsupported")
            );
            continue;
        }
        assert_eq!(result.is_ok(), accepted, "{}: {result:?}", row["id"]);
        if let Ok(sdf) = result {
            let mat = sdf.models[0].links[0].visuals[0].material.as_deref();
            assert_eq!(mat.is_some(), tokens[1] == "1", "{}", row["id"]);
            if let Some(mat) = mat {
                let mut i = 2;
                for c in [mat.ambient, mat.diffuse, mat.specular, mat.emissive] {
                    for v in c {
                        assert_eq!(
                            u64::from(v.to_bits()),
                            tokens[i].as_str().unwrap().parse::<u64>().unwrap(),
                            "{} channel{i}",
                            row["id"]
                        );
                        i += 1;
                    }
                }
                assert_eq!(
                    mat.shininess.to_bits(),
                    tokens[i].as_str().unwrap().parse::<u64>().unwrap()
                );
                i += 1;
                assert_eq!(
                    u64::from(mat.render_order.to_bits()),
                    tokens[i].as_str().unwrap().parse::<u64>().unwrap()
                );
                i += 1;
                assert_eq!(mat.lighting, tokens[i] == "1");
                i += 1;
                assert_eq!(mat.double_sided, tokens[i] == "1");
                i += 1;
                let shader = match mat.shader {
                    ShaderType::Pixel => 0,
                    ShaderType::Vertex => 1,
                    ShaderType::NormalMapObjectSpace => 2,
                    ShaderType::NormalMapTangentSpace => 3,
                };
                assert_eq!(shader, tokens[i].as_str().unwrap().parse::<u32>().unwrap());
                i += 1;
                assert_eq!(mat.normal_map, text(&tokens[i]));
                i += 1;
                assert_eq!(
                    mat.script
                        .as_ref()
                        .map(|s| s.uris[0].as_str())
                        .unwrap_or(""),
                    text(&tokens[i])
                );
                i += 1;
                assert_eq!(
                    mat.script.as_ref().map(|s| s.name.as_str()).unwrap_or(""),
                    text(&tokens[i])
                );
            }
        }
    }
}

#[test]
fn material_script_metadata_and_source_survive_document_drop() {
    let owned = {
        let sdf=load_str("<sdf version='1.12'><model name='m'><link name='l'><visual name='v'>\n<material><script><uri>first</uri><uri/><uri>third</uri><name>script</name></script></material></visual></link></model></sdf>","cached/model.sdf",&NoResources).unwrap();
        sdf.models[0].links[0].visuals[0]
            .material
            .as_deref()
            .unwrap()
            .clone()
    };
    assert_eq!(owned.source.uri, "cached/model.sdf");
    assert_eq!(owned.source.line, 2);
    assert_eq!(owned.source.column, 1);
    let script = owned.script.unwrap();
    assert_eq!(script.uris, ["first", "__default__", "third"]);
    assert_eq!(script.name, "script");
    assert_eq!(script.source.uri, "cached/model.sdf");
}
