// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
#[path = "common/auto_default_density.rs"]
mod common;

fn values(row: &serde_json::Value) -> Vec<u64> {
    let lines: Vec<_> = row["stdout"]
        .as_str()
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("VALUES "))
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], lines[1]);
    lines[0]
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect()
}

#[test]
fn native_default_density_values_keep_explicit_presence_and_source() {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("auto_default_density.json")).unwrap();
    let mut positives = 0;
    let mut negatives = 0;
    // The four strict native warning-policy rows have no Rust policy equivalent.
    // The native replay executes them; this differential uses the default policy.
    for row in data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["strict_warning"] == false)
    {
        let xml = row["xml"].as_str().unwrap();
        let uri = row["uri"].as_str().unwrap();
        if row.get("rust_error").is_some() {
            assert_eq!(row["rust_error"], "InvalidValue");
            let error = load_str(xml, uri, &NoResources).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidValue);
            assert_eq!(error.source.uri, uri);
            assert!(error.source.line > 0 && error.source.column > 0);
            negatives += 1;
            continue;
        }
        let native: Vec<_> = row["stdout"].as_str().unwrap().lines().collect();
        assert_eq!(native[0], "LOAD");
        let presence = row["density_presence"].as_bool().unwrap();
        let density: Vec<_> = native
            .iter()
            .filter_map(|line| line.strip_prefix("DENSITY "))
            .collect();
        assert_eq!(density.len(), 2);
        assert_eq!(density[0], density[1]);
        assert_eq!(
            density[0],
            format!(
                "{} {} {}",
                u8::from(presence),
                1000_f64.to_bits(),
                Collision::DENSITY_DEFAULT.to_bits()
            )
        );
        let version = xml
            .strip_prefix("<sdf version='")
            .unwrap()
            .split('\'')
            .next()
            .unwrap();
        common::assert_complete_default_density(
            xml,
            uri,
            version,
            row["shape"].as_str().unwrap(),
            presence.then_some(1000.),
            &values(row),
        );
        positives += 1;
    }
    assert_eq!((positives, negatives), (74, 8));
}

#[test]
fn included_default_density_preserves_absence_frames_and_resource_identity() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!(uri, "box");
            assert_eq!(parent, "world.sdf");
            let data: serde_json::Value =
                serde_json::from_str(include_str!("auto_box_inertial.json")).unwrap();
            let xml = data["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["name"] == "default_density")
                .unwrap()["xml"]
                .as_str()
                .unwrap()
                .to_owned();
            Ok(Resource {
                canonical_uri: "boxes/default-box.sdf".into(),
                xml,
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>box</uri><name>imported</name><pose>1 2 3 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    let link = &model.links[0];
    let collision = &link.collisions[0];
    assert_eq!(model.name, "imported");
    for source in [&model.source, &link.source, &collision.source] {
        assert_eq!(source.uri, "boxes/default-box.sdf");
    }
    assert_eq!(collision.density, None);
    assert_eq!(collision.density(), 1000.);
    assert_eq!(link.inertial.mass, 8000.);
    assert_eq!(link.inertial.pose, Pose::default());
    assert_eq!(
        sdf.pose_of("w::imported::link", "::").unwrap().translation,
        [1., 2., 3.]
    );
    sdf.validate_non_frame_poses().unwrap();
    let mut copy = model.clone();
    copy.links[0].collisions[0].density = Some(5.);
    assert_eq!(collision.density, None);
    assert_eq!(link.inertial.mass, 8000.);
}
