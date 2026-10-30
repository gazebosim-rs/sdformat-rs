// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{Mesh, MeshOptimization};
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
pub fn projection(value: &Mesh) -> Vec<String> {
    let (name, kind) = match value.optimization {
        MeshOptimization::None => ("", 0),
        MeshOptimization::ConvexHull => ("convex_hull", 1),
        MeshOptimization::ConvexDecomposition => ("convex_decomposition", 2),
    };
    let mut out = vec![
        "VALUES".into(),
        hex(&value.uri),
        hex(name),
        kind.to_string(),
    ];
    out.extend(value.scale.map(|v| v.to_bits().to_string()));
    out.extend([
        hex(&value.submesh),
        u8::from(value.center_submesh).to_string(),
        u8::from(value.convex_decomposition.is_some()).to_string(),
    ]);
    if let Some(v) = value.convex_decomposition {
        out.extend([
            v.max_convex_hulls.to_string(),
            v.voxel_resolution.to_string(),
        ]);
    }
    out
}
pub fn expected(stdout: &str) -> Vec<String> {
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(&lines[..2], ["READ 1", "LOAD"]);
    assert_eq!(lines.len(), 3);
    let tokens: Vec<_> = lines[2].split_whitespace().map(str::to_owned).collect();
    assert_eq!(tokens.len(), if tokens[9] == "1" { 12 } else { 10 });
    tokens
}

pub fn assert_geometry(xml: &str, uri: &str, native: &str) {
    use sdformat::*;
    let geometry = if xml.starts_with("<sdf") {
        let d = load_str(xml, uri, &NoResources).unwrap();
        assert_eq!(d.models.len(), 1);
        assert_eq!(d.models[0].name, "m");
        assert_eq!(d.models[0].links.len(), 1);
        assert_eq!(d.models[0].links[0].name, "l");
        assert_eq!(d.models[0].links[0].collisions.len(), 1);
        assert_eq!(d.models[0].links[0].collisions[0].name, "c");
        d.models[0].links[0].collisions[0].geometry.clone()
    } else {
        load_geometry_str(xml, uri).unwrap()
    };
    let Geometry::Mesh(mesh) = &geometry else {
        panic!("not Mesh")
    };
    let expected = expected(native);
    assert_eq!(projection(mesh), expected);
    assert_eq!(mesh.source.uri, uri);
    let before = geometry.clone();
    let output = geometry.to_xml().unwrap();
    assert_eq!(geometry, before);
    let Geometry::Mesh(reloaded) = load_geometry_str(&output, "roundtrip.xml").unwrap() else {
        panic!("not Mesh")
    };
    assert_eq!(projection(&reloaded), expected);
}
