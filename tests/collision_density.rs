// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

fn document(fragment: &str) -> String {
    format!(
        "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'>{fragment}</collision></link></model></sdf>"
    )
}
fn load(fragment: &str) -> Result<Sdf, Diagnostic> {
    load_str(&document(fragment), "density.sdf", &NoResources)
}
fn collision(sdf: &Sdf) -> &Collision {
    &sdf.models[0].links[0].collisions[0]
}
#[test]
fn native_density_presence_and_values() {
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("collision_density.json")).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 14);
    let mut checked = 0;
    for row in rows.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        if [
            "absent",
            "empty",
            "configured",
            "default",
            "minus_zero",
            "negative",
            "prefix",
        ]
        .contains(&id)
        {
            checked += 1;
            let sdf = load(xml).unwrap();
            let value = collision(&sdf);
            let oracle = row["stdout"].as_str().unwrap();
            let bits: u64 = oracle
                .lines()
                .find_map(|line| line.strip_prefix("VALUE "))
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(value.density().to_bits(), bits, "{id}");
            assert_eq!(
                value.density.is_some(),
                oracle.contains("PRESENT 1\n"),
                "{id}"
            );
            if let Some(density) = value.density {
                assert_eq!(density.to_bits(), bits);
            }
            assert_eq!(value.name, "c");
            assert_eq!(value.geometry, Geometry::Empty);
            assert_eq!(value.pose, Pose::default());
            assert_eq!(value.surface, Surface::default());
            assert_eq!(
                value.source,
                Source {
                    uri: "density.sdf".into(),
                    line: 1,
                    column: 52
                }
            );
            assert_eq!(sdf.pose_of("m::l", "::").unwrap(), Transform::default());
        }
    }
    assert_eq!(checked, 7);
}
#[test]
fn density_edits_and_clones_preserve_presence_and_bits() {
    let sdf = load("").unwrap();
    let mut value = collision(&sdf).clone();
    assert_eq!(Collision::DENSITY_DEFAULT, 1000.);
    assert_eq!(value.density, None);
    assert_eq!(value.density(), 1000.);
    for density in [
        0.,
        -0.,
        -1.,
        1234.5,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
        f64::NAN,
        f64::INFINITY,
    ] {
        value.density = Some(density);
        assert_eq!(value.density().to_bits(), density.to_bits());
        let mut copy = value.clone();
        assert_eq!(copy.density.unwrap().to_bits(), density.to_bits());
        copy.density = None;
        assert_eq!(copy.density(), 1000.);
        assert_eq!(value.density.unwrap().to_bits(), density.to_bits());
    }
    assert_eq!(collision(&sdf).density, None);
}
#[test]
fn existing_scalar_and_structure_policies_remain_strict() {
    for (xml, code) in [
        ("<density>nan</density>", ErrorCode::InvalidValue),
        ("<density>inf</density>", ErrorCode::InvalidValue),
        (
            "<density>4.9406564584124654e-324</density>",
            ErrorCode::InvalidValue,
        ),
        ("<density>bad</density>", ErrorCode::InvalidValue),
        (
            "<density>1</density><density>2</density>",
            ErrorCode::InvalidValue,
        ),
        ("<density unknown='x'>1</density>", ErrorCode::Unsupported),
        ("<density><unknown/></density>", ErrorCode::Unsupported),
    ] {
        let error = load(xml).unwrap_err();
        assert_eq!(error.code, code, "{xml}");
        assert_eq!(error.source.uri, "density.sdf");
        assert_eq!(error.source.line, 1);
        assert!(error.source.column >= 65);
    }
}
