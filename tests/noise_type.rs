// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, NoResources, NoiseKind, load_str};
fn cases() -> serde_json::Value {
    serde_json::from_str(include_str!("noise_type.json")).unwrap()
}
#[test]
fn native_noise_type_root_corpus() {
    let cases = cases();
    assert_eq!(cases.as_array().unwrap().len(), 126);
    for row in cases.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let xml = row["xml"].as_str().unwrap();
        let parsed = load_str(xml, "noise-type.sdf", &NoResources);
        let tokens = row["tokens"].as_array().unwrap();
        if tokens[0] != "0" {
            let error = parsed.unwrap_err();
            let code = match row["rust_error"].as_str().unwrap() {
                "Missing" => ErrorCode::Missing,
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unqualified error {id}"),
            };
            assert_eq!(error.code, code, "{id}");
            if let Some(node) = row.get("source_node") {
                assert_eq!(error.source.line, 1);
                assert_eq!(
                    error.source.column,
                    u32::try_from(xml.find(node.as_str().unwrap()).unwrap() + 1).unwrap(),
                    "{id}"
                );
            }
            continue;
        }
        let sdf = parsed.unwrap_or_else(|error| panic!("{id}: {error}"));
        let imu = sdf.models[0].links[0].sensors[0].imu().unwrap();
        let mut expected = tokens.iter().skip(1).map(|v| v.as_str().unwrap());
        assert_eq!(expected.next(), Some("1"), "{id}");
        for noise in imu.angular_velocity.iter().chain(&imu.linear_acceleration) {
            let kind = match noise.kind {
                NoiseKind::None => 0,
                NoiseKind::Gaussian => 1,
                NoiseKind::GaussianQuantized => 2,
            };
            assert_eq!(
                kind,
                expected.next().unwrap().parse::<u8>().unwrap(),
                "{id}"
            );
            for value in [
                noise.mean,
                noise.stddev,
                noise.bias_mean,
                noise.bias_stddev,
                noise.dynamic_bias_stddev,
                noise.dynamic_bias_correlation_time,
                noise.precision,
            ] {
                assert_eq!(
                    value.to_bits(),
                    expected.next().unwrap().parse::<u64>().unwrap(),
                    "{id}"
                );
            }
        }
        assert!(expected.next().is_none(), "{id}");
    }
}
#[test]
fn former_supplementary_positive_is_exact_required_type_rejection() {
    let cases = cases();
    let row = cases
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "former_supplementary_positive")
        .unwrap();
    let disposition: serde_json::Value =
        serde_json::from_str(include_str!("noise_type_disposition.json")).unwrap();
    assert_eq!(row["xml"], disposition["exact_old_xml"]);
    assert_ne!(row["tokens"][0], "0");
    assert_eq!(
        load_str(row["xml"].as_str().unwrap(), "old-positive", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Missing
    );
}
#[test]
fn native_ascii_noise_type_padding_is_accepted() {
    let cases = cases();
    for id in ["1.12_space", "1.12_tab_ref", "type_ref_ws_000A"] {
        let row = cases
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        assert_eq!(row["tokens"][0], "0");
        let sdf = load_str(row["xml"].as_str().unwrap(), "trim", &NoResources).unwrap();
        assert_eq!(
            sdf.models[0].links[0].sensors[0]
                .imu()
                .unwrap()
                .angular_velocity[0]
                .kind,
            NoiseKind::Gaussian
        );
    }
}
