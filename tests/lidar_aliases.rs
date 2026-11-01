/*
 * Copyright (C) 2019 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, Lidar, NoResources, Source, load_lidar_str, load_str};
use serde_json::Value;

fn values(lidar: &Lidar) -> Vec<u64> {
    let mut values = Vec::new();
    for scan in [&lidar.horizontal, &lidar.vertical] {
        values.push(u64::from(scan.samples));
        values.extend([scan.resolution, scan.min_angle, scan.max_angle].map(f64::to_bits));
    }
    values.extend([lidar.range_min, lidar.range_max, lidar.range_resolution].map(f64::to_bits));
    values.push(u64::from(lidar.visibility_mask));
    values.push(match lidar.noise.kind {
        sdformat::NoiseKind::None => 0,
        sdformat::NoiseKind::Gaussian => 1,
        sdformat::NoiseKind::GaussianQuantized => 2,
    });
    values.extend(
        [
            lidar.noise.mean,
            lidar.noise.stddev,
            lidar.noise.bias_mean,
            lidar.noise.bias_stddev,
            lidar.noise.dynamic_bias_stddev,
            lidar.noise.dynamic_bias_correlation_time,
            lidar.noise.precision,
        ]
        .map(f64::to_bits),
    );
    values
}

#[test]
fn native_standalone_aliases_preserve_complete_values_and_input_source() {
    let corpus: Value = serde_json::from_str(include_str!("lidar_aliases.json")).unwrap();
    let mut admitted = 0;
    for row in corpus["cases"].as_array().unwrap() {
        if row.get("rust_error").is_some() {
            continue;
        }
        admitted += 1;
        let name = row["name"].as_str().unwrap();
        let native = row["native"]["stdout"].as_str().unwrap();
        assert_eq!(
            native.lines().take(3).collect::<Vec<_>>(),
            ["READ 1", "SCHEMA", "LOAD"]
        );
        assert_eq!(row["native"]["stderr"], "");
        let expected = native
            .lines()
            .find_map(|line| line.strip_prefix("VALUES "))
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse::<u64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(expected.len(), 20);
        let uri = format!("caller:{name}");
        let lidar = load_lidar_str(row["xml"].as_str().unwrap(), &uri).unwrap();
        assert_eq!(values(&lidar), expected, "{name}");
        assert_eq!(
            lidar.source,
            Some(Source {
                uri,
                line: 2,
                column: 3
            })
        );
        let mut copied = lidar.clone();
        copied.horizontal.samples = copied.horizontal.samples.wrapping_add(1);
        assert_ne!(copied.horizontal.samples, lidar.horizontal.samples);
        assert_eq!(copied.source, lidar.source);
        let before = lidar.clone();
        let canonical = lidar.to_xml().unwrap();
        assert!(canonical.starts_with("<lidar>"));
        assert_eq!(
            values(&load_lidar_str(&canonical, "output.xml").unwrap()),
            expected
        );
        assert_eq!(lidar, before);
    }
    assert_eq!(admitted, 36);
}

#[test]
fn alias_admission_preserves_required_groups_and_strict_subset_errors() {
    let corpus: Value = serde_json::from_str(include_str!("lidar_aliases.json")).unwrap();
    let mut rejected = 0;
    for row in corpus["cases"].as_array().unwrap() {
        let Some(error) = row.get("rust_error") else {
            continue;
        };
        rejected += 1;
        let expected = match error.as_str().unwrap() {
            "Missing" => ErrorCode::Missing,
            "Unsupported" => ErrorCode::Unsupported,
            "InvalidValue" => ErrorCode::InvalidValue,
            _ => panic!("unexpected diagnostic"),
        };
        let error = load_lidar_str(row["xml"].as_str().unwrap(), "strict.xml").unwrap_err();
        assert_eq!(error.code, expected, "{}: {error}", row["name"]);
        assert_eq!(error.source.uri, "strict.xml");
        assert_eq!(error.source.line, 2);
        assert!(error.source.column >= 3);
    }
    assert_eq!(rejected, 28);
    for tag in ["lidar", "ray", "gpu_ray", "gpu_lidar"] {
        for content in [
            format!("<{tag} unknown='x'><scan><horizontal/></scan><range/></{tag}>"),
            format!(
                "<{tag}><scan><horizontal><min_angle degrees='true'>1</min_angle></horizontal></scan><range/></{tag}>"
            ),
            format!("<{tag}><scan><horizontal/></scan><range/><noise type='gaussian'/></{tag}>"),
        ] {
            assert_eq!(
                load_lidar_str(&content, "strict.xml").unwrap_err().code,
                ErrorCode::Unsupported
            );
        }
    }
}

#[test]
fn original_wrong_element_failure_remains_explicit_without_mutable_element_claim() {
    // Lidar_TEST.cc:115-121 uses an Element named "bad". This value reader has
    // no null Element or retained mutable pointer, so those observations stay unported.
    let error = load_lidar_str("<bad/>", "bad.xml").unwrap_err();
    assert_eq!(error.code, ErrorCode::Xml);
    assert_eq!(
        error.source,
        Source {
            uri: "bad.xml".into(),
            line: 1,
            column: 1
        }
    );
    for xml in ["", "<", "<Lidar/>", "<Ray/>", "<gpu-ray/>", "<gpuLidar/>"] {
        assert_eq!(
            load_lidar_str(xml, "wrong.xml").unwrap_err().code,
            ErrorCode::Xml
        );
    }
}

#[test]
fn standalone_aliases_do_not_broaden_document_sensor_payloads() {
    for version in ["1.4", "1.5", "1.6", "1.8", "1.12"] {
        for kind in ["lidar", "ray", "gpu_lidar", "gpu_ray"] {
            for tag in ["gpu_lidar", "gpu_ray"] {
                let xml = format!(
                    "<sdf version='{version}'><model name='m'><link name='l'><sensor name='s' type='{kind}'><{tag}><scan><horizontal/></scan><range/></{tag}></sensor></link></model></sdf>"
                );
                assert_eq!(
                    load_str(&xml, "document.sdf", &NoResources)
                        .unwrap_err()
                        .code,
                    ErrorCode::Unsupported
                );
            }
        }
    }
}
