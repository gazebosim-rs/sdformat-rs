// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
fn hex(value: &str) -> String {
    if value.is_empty() {
        return "-".into();
    }
    value
        .as_bytes()
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect()
}
pub fn projection(value: &Material) -> Vec<Vec<String>> {
    let mut base = vec!["BASE".into()];
    base.extend(
        [value.ambient, value.diffuse, value.specular, value.emissive]
            .into_iter()
            .flatten()
            .map(|v| v.to_bits().to_string()),
    );
    base.extend([
        value.shininess.to_bits().to_string(),
        value.render_order.to_bits().to_string(),
        u8::from(value.lighting).to_string(),
        u8::from(value.double_sided).to_string(),
        match value.shader {
            ShaderType::Pixel => 0,
            ShaderType::Vertex => 1,
            ShaderType::NormalMapObjectSpace => 2,
            ShaderType::NormalMapTangentSpace => 3,
        }
        .to_string(),
        hex(&value.normal_map),
        hex(value.script.as_ref().map_or("", |s| s.uris[0].as_str())),
        hex(value.script.as_ref().map_or("", |s| s.name.as_str())),
    ]);
    let mut raw = vec![
        "RAW_SCRIPT".into(),
        value
            .script
            .as_ref()
            .map_or(0, |s| s.uris.len())
            .to_string(),
    ];
    if let Some(script) = &value.script {
        raw.extend(script.uris.iter().map(|s| hex(s)));
    }
    let mut lines = vec![
        base,
        raw,
        vec!["PBR".into(), u8::from(value.pbr.is_some()).to_string()],
    ];
    for flow in [
        value.pbr.as_ref().and_then(|p| p.metal.as_ref()),
        value.pbr.as_ref().and_then(|p| p.specular.as_ref()),
    ] {
        let mut fields = vec!["FLOW".into(), u8::from(flow.is_some()).to_string()];
        if let Some(f) = flow {
            fields.push(
                match f.kind {
                    PbrWorkflowKind::Metal => 1,
                    PbrWorkflowKind::Specular => 2,
                }
                .to_string(),
            );
            for path in [
                &f.albedo_map,
                &f.normal_map,
                &f.environment_map,
                &f.ambient_occlusion_map,
                &f.emissive_map,
                &f.light_map,
                &f.roughness_map,
                &f.metalness_map,
                &f.specular_map,
                &f.glossiness_map,
            ] {
                fields.push(hex(path));
            }
            // The pinned native loader leaves the getter at Tangent; the raw
            // attribute has a separate immutable metadata contract below.
            fields.extend([
                "0".into(),
                f.light_map_uv_set.to_string(),
                f.roughness.to_bits().to_string(),
                f.metalness.to_bits().to_string(),
                f.glossiness.to_bits().to_string(),
            ]);
        }
        lines.push(fields);
    }
    lines
}
pub fn raw_types(value: &Material) -> Vec<Option<String>> {
    [
        value.pbr.as_ref().and_then(|p| p.metal.as_ref()),
        value.pbr.as_ref().and_then(|p| p.specular.as_ref()),
    ]
    .into_iter()
    .map(|v| v.and_then(|f| f.normal_map_type.clone()))
    .collect()
}
pub fn sources(value: &Material) -> Vec<Source> {
    let mut out = vec![value.source.clone()];
    if let Some(s) = &value.script {
        out.push(s.source.clone());
    }
    if let Some(p) = &value.pbr {
        out.push(p.source.clone());
        for f in p.metal.iter().chain(&p.specular) {
            out.push(f.source.clone());
        }
    }
    out
}
pub fn expected(stdout: &str) -> Vec<Vec<String>> {
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 7);
    assert_eq!(&lines[..2], ["READ 1", "LOAD"]);
    let lines: Vec<Vec<String>> = lines[2..]
        .iter()
        .map(|s| s.split_whitespace().map(str::to_owned).collect())
        .collect();
    assert_eq!(lines[0].len(), 25);
    assert_eq!(lines[0][0], "BASE");
    assert_eq!(lines[1][0], "RAW_SCRIPT");
    assert_eq!(lines[1].len(), 2 + lines[1][1].parse::<usize>().unwrap());
    assert_eq!(lines[2].len(), 2);
    assert_eq!(lines[2][0], "PBR");
    for f in &lines[3..] {
        assert_eq!(f[0], "FLOW");
        assert_eq!(f.len(), if f[1] == "1" { 18 } else { 2 });
    }
    lines
}
