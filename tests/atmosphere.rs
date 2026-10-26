// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{AtmosphereType, ErrorCode, NoResources, load_str};
use serde_json::Value;

#[test]
fn native_world_atmosphere_metadata_and_explicit_gaps() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("atmosphere.json")).unwrap();
    for row in rows {
        let result = load_str(row["xml"].as_str().unwrap(), "atmosphere.sdf", &NoResources);
        if let Some(gap) = row.get("gap_code") {
            assert_eq!(row["native_accepted"], true);
            let expected = match gap.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                "Missing" => ErrorCode::Missing,
                _ => panic!("unmapped explicit atmosphere gap"),
            };
            assert_eq!(result.unwrap_err().code, expected, "{}", row["id"]);
            continue;
        }
        assert_eq!(
            result.is_ok(),
            row["native_accepted"].as_bool().unwrap(),
            "{}: {result:?}",
            row["id"]
        );
        if let Ok(sdf) = result {
            let a = &sdf.worlds[0].atmosphere;
            let tokens = row["tokens"].as_array().unwrap();
            assert_eq!(tokens[1], "1");
            assert_eq!(tokens[2], "adiabatic");
            assert_eq!(a.atmosphere_type, AtmosphereType::Adiabatic);
            for (actual, token) in [a.temperature_kelvin, a.pressure, a.temperature_gradient]
                .into_iter()
                .zip(&tokens[3..])
            {
                assert_eq!(
                    actual.to_bits(),
                    token.as_str().unwrap().parse::<u64>().unwrap(),
                    "{}",
                    row["id"]
                );
            }
        }
    }
}

#[test]
fn atmosphere_owns_explicit_provenance_and_marks_implicit_defaults() {
    let xml = "<atmosphere type='adiabatic'><temperature>23.1</temperature><pressure>43.1</pressure><temperature_gradient>4.3</temperature_gradient></atmosphere>";
    let text = format!("<sdf version='1.12'><world name='w'>{xml}</world></sdf>");
    let sdf = load_str(&text, "world.sdf", &NoResources).unwrap();
    let a = sdf.worlds[0].atmosphere.clone();
    drop(sdf);
    assert_eq!(a.xml.as_deref(), Some(xml));
    assert_eq!(a.source.as_ref().unwrap().uri, "world.sdf");
    assert_eq!(a.source.as_ref().unwrap().line, 1);
    assert_eq!(a.temperature_kelvin, 23.1);
    let sdf = load_str(
        "<sdf version='1.12'><world name='w'/></sdf>",
        "default.sdf",
        &NoResources,
    )
    .unwrap();
    let a = &sdf.worlds[0].atmosphere;
    assert!(a.source.is_none());
    assert!(a.xml.is_none());
    assert_eq!(a.temperature_kelvin, 288.15);
}
