// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
pub fn projection(lines: &[Polyline]) -> Vec<String> {
    let mut out = vec!["VALUES".into(), lines.len().to_string()];
    for line in lines {
        out.extend([
            line.height.to_bits().to_string(),
            line.points.len().to_string(),
        ]);
        out.extend(
            line.points
                .iter()
                .flatten()
                .map(|v| v.to_bits().to_string()),
        );
    }
    out
}
pub fn expected(stdout: &str) -> Vec<String> {
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(&lines[..2], ["READ 1", "LOAD"]);
    let tokens: Vec<_> = lines[2].split_whitespace().map(str::to_owned).collect();
    assert!(tokens.len() >= 2);
    let count: usize = tokens[1].parse().unwrap();
    let mut index = 2;
    for _ in 0..count {
        assert!(tokens.len() >= index + 2);
        let points: usize = tokens[index + 1].parse().unwrap();
        index += 2 + 2 * points;
        assert!(tokens.len() >= index);
    }
    assert_eq!(tokens.len(), index);
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
    let Geometry::Polyline(lines) = &geometry else {
        panic!("not Polyline")
    };
    let expected = expected(native);
    assert_eq!(projection(lines), expected);
    assert!(lines.iter().all(|v| v.source.uri == uri));
    let before = geometry.clone();
    let output = geometry.to_xml().unwrap();
    assert_eq!(geometry, before);
    let Geometry::Polyline(reloaded) = load_geometry_str(&output, "roundtrip.xml").unwrap() else {
        panic!("not Polyline")
    };
    assert_eq!(projection(&reloaded), expected);
}
