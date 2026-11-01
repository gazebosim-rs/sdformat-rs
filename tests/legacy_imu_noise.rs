/*
 * Copyright 2012 Open Source Robotics Foundation
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
use sdformat::*;
use serde_json::Value;

fn sensor(sdf: &Sdf) -> &Sensor {
    let models = if sdf.worlds.is_empty() {
        &sdf.models
    } else {
        &sdf.worlds[0].models
    };
    &models[0].links[0].sensors[0]
}
fn noise_values(imu: &Imu) -> Vec<u64> {
    imu.angular_velocity
        .iter()
        .chain(&imu.linear_acceleration)
        .flat_map(|noise| {
            let kind = match noise.kind {
                NoiseKind::None => 0,
                NoiseKind::Gaussian => 1,
                NoiseKind::GaussianQuantized => 2,
            };
            std::iter::once(kind).chain(
                [
                    noise.mean,
                    noise.stddev,
                    noise.bias_mean,
                    noise.bias_stddev,
                    noise.dynamic_bias_stddev,
                    noise.dynamic_bias_correlation_time,
                    noise.precision,
                ]
                .map(f64::to_bits),
            )
        })
        .collect()
}
fn compare_native(sdf: &Sdf, native: &str) {
    assert_eq!(native.lines().next(), Some("ERRORS"));
    let expected = native
        .lines()
        .find_map(|s| s.strip_prefix("NOISE "))
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<u64>().unwrap())
        .collect::<Vec<_>>();
    let imu = sensor(sdf).imu().unwrap();
    assert_eq!(expected.len(), 48);
    assert_eq!(noise_values(imu), expected);
    let hex = |value: &str| {
        if value.is_empty() {
            "-".into()
        } else {
            value
                .bytes()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        }
    };
    let mut orientation = vec![
        u8::from(imu.enable_orientation).to_string(),
        hex(&imu.localization),
    ];
    orientation.extend(imu.custom_rpy.map(|value| value.to_bits().to_string()));
    orientation.push(hex(&imu.custom_rpy_parent_frame));
    orientation.extend(imu.gravity_dir_x.map(|value| value.to_bits().to_string()));
    orientation.push(hex(&imu.gravity_dir_x_parent_frame));
    assert_eq!(
        orientation,
        native
            .lines()
            .find_map(|line| line.strip_prefix("ORIENTATION "))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
    );
    let sensor = sensor(sdf);
    let envelope = [
        hex(&sensor.topic),
        hex(&sensor.frame_id),
        sensor.update_rate.to_bits().to_string(),
        u8::from(sensor.enable_metrics).to_string(),
        u8::from(sensor.always_on).to_string(),
        u8::from(sensor.visualize).to_string(),
    ];
    assert_eq!(
        envelope,
        native
            .lines()
            .find_map(|line| line.strip_prefix("ENVELOPE "))
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .as_slice()
    );
}
struct Resources;
impl ResourceResolver for Resources {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        assert_eq!(uri, "fixture.sdf");
        assert_eq!(parent, "legacy.sdf");
        Ok(Resource {
            canonical_uri: "fixture:legacy_imu.sdf".into(),
            xml: include_str!("fixtures/legacy_imu_resource.sdf").into(),
        })
    }
}
#[test]
fn pinned_legacy_imu_controls_and_original_provenance() {
    let data: Value = serde_json::from_str(include_str!("legacy_imu_noise.json")).unwrap();
    let mut admitted = 0;
    for row in data["cases"].as_array().unwrap() {
        let xml = row["xml"].as_str().unwrap();
        let result = load_str(xml, "legacy.sdf", &Resources);
        if let Some(error) = row.get("rust_error") {
            let expected = match error.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "Missing" => ErrorCode::Missing,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown error"),
            };
            let error = result.unwrap_err();
            assert_eq!(error.code, expected, "{}: {error}", row["name"]);
            assert_eq!(error.source.uri, "legacy.sdf");
            continue;
        }
        admitted += 1;
        let sdf = result.unwrap_or_else(|error| panic!("{}: {error}", row["name"]));
        compare_native(&sdf, row["native"]["stdout"].as_str().unwrap());
        assert_eq!(row["native"]["stderr"], "");
        let origin = if row.get("resource").is_some() {
            "fixture:legacy_imu.sdf"
        } else {
            "legacy.sdf"
        };
        assert_eq!(sensor(&sdf).source.uri, origin);
        assert_eq!(
            sensor(&sdf).imu().unwrap().source.as_ref().unwrap().uri,
            origin
        );
        assert!(sensor(&sdf).configuration_xml.contains("<noise>"));
        assert!(sensor(&sdf).configuration_xml.contains("<type>"));
        assert!(
            !sensor(&sdf)
                .configuration_xml
                .contains("<angular_velocity>")
        );
        assert!(
            !sensor(&sdf)
                .configuration_xml
                .contains("<linear_acceleration>")
        );
        assert_eq!(
            sdf.version,
            if row.get("resource").is_some() {
                "1.12"
            } else if xml.contains("version=\"1.4\"") {
                "1.4"
            } else {
                "1.5"
            }
        );
        sdf.validate_non_frame_poses().unwrap();
        let before = sensor(&sdf).clone();
        let mut clone = before.clone();
        let SensorConfig::Imu(copy) = &mut clone.config else {
            panic!("IMU")
        };
        copy.angular_velocity[0].mean += 1.;
        assert_ne!(format!("{clone:?}"), format!("{before:?}"));
        assert_eq!(format!("{:?}", sensor(&sdf)), format!("{before:?}"));
        let canonical = before.to_xml().unwrap();
        let mut loaded = Sensor::default();
        loaded
            .load_element_str(&canonical, "canonical.xml", &NoResources)
            .unwrap();
        assert_eq!(
            noise_values(loaded.imu().unwrap()),
            noise_values(before.imu().unwrap())
        );
    }
    assert_eq!(admitted, 42);
}
#[test]
fn original_converter_imu_noise_values_keep_original_xml_version_and_source() {
    let xml = include_str!("fixtures/legacy_imu_original.sdf");
    let sdf = load_str(xml, "original-imu.sdf", &NoResources).unwrap();
    let data: Value = serde_json::from_str(include_str!("legacy_imu_noise.json")).unwrap();
    compare_native(&sdf, data["original"]["stdout"].as_str().unwrap());
    assert_eq!(sdf.version, "1.5");
    assert!(sdf.models.is_empty());
    assert_eq!(sdf.worlds.len(), 1);
    assert_eq!(sdf.worlds[0].name, "default");
    assert_eq!(sdf.worlds[0].models[0].name, "box_old_imu_noise");
    assert_eq!(sdf.worlds[0].models[0].links[0].name, "link");
    let sensor = sensor(&sdf);
    assert_eq!(sensor.name, "imu_sensor");
    assert_eq!(sensor.kind, "imu");
    assert!(xml.contains(&sensor.configuration_xml));
    assert!(sensor.configuration_xml.contains("<noise>"));
    for text in [
        "<stddev>0.0002</stddev>",
        "<bias_mean>7.5e-06</bias_mean>",
        "<bias_stddev>8e-07</bias_stddev>",
        "<stddev>0.017</stddev>",
        "<bias_mean>0.1</bias_mean>",
        "<bias_stddev>0.001</bias_stddev>",
    ] {
        assert!(sensor.configuration_xml.contains(text));
    }
    assert_eq!(
        sensor.imu().unwrap().source,
        Some(Source {
            uri: "original-imu.sdf".into(),
            line: 8,
            column: 11
        })
    );
}
#[test]
fn standalone_legacy_input_stays_unsupported_without_document_conversion_context() {
    let imu = "<imu><noise><type>gaussian</type></noise></imu>";
    assert_eq!(
        load_imu_str(imu, "standalone.xml").unwrap_err().code,
        ErrorCode::Unsupported
    );
    let mut sensor = Sensor::default();
    assert_eq!(
        sensor
            .load_element_str(
                &format!("<sensor name='s' type='imu'>{imu}</sensor>"),
                "standalone.xml",
                &NoResources
            )
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}
