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
pub fn projection(value: &Heightmap) -> Vec<String> {
    let mut out = vec!["VALUES".into(), hex(&value.uri)];
    out.extend(
        value
            .size
            .into_iter()
            .chain(value.position)
            .map(|v| v.to_bits().to_string()),
    );
    out.extend([
        u8::from(value.use_terrain_paging).to_string(),
        value.sampling.to_string(),
        value.textures.len().to_string(),
    ]);
    for t in &value.textures {
        out.extend([
            t.size.to_bits().to_string(),
            hex(&t.diffuse),
            hex(&t.normal),
        ]);
    }
    out.push(value.blends.len().to_string());
    for b in &value.blends {
        out.extend([
            b.min_height.to_bits().to_string(),
            b.fade_distance.to_bits().to_string(),
        ]);
    }
    out
}
pub fn expected(stdout: &str) -> Vec<String> {
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(&lines[..2], ["READ 1", "LOAD"]);
    let tokens: Vec<_> = lines[2].split_whitespace().map(str::to_owned).collect();
    assert!(tokens.len() >= 12);
    let textures: usize = tokens[10].parse().unwrap();
    let blend_index = 11 + 3 * textures;
    assert!(tokens.len() > blend_index);
    let blends: usize = tokens[blend_index].parse().unwrap();
    assert_eq!(tokens.len(), blend_index + 1 + 2 * blends);
    tokens
}
pub fn assert_geometry(xml: &str, uri: &str, native: &str) {
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
    let Geometry::Heightmap(value) = &geometry else {
        panic!("not Heightmap")
    };
    let expected = expected(native);
    assert_eq!(projection(value), expected);
    assert_eq!(value.source.uri, uri);
    let before = geometry.clone();
    let output = geometry.to_xml().unwrap();
    assert_eq!(geometry, before);
    let Geometry::Heightmap(reloaded) = load_geometry_str(&output, "roundtrip.xml").unwrap() else {
        panic!("not Heightmap")
    };
    assert_eq!(projection(&reloaded), expected);
}
