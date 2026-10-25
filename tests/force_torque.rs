/*
 * Copyright 2018 Open Source Robotics Foundation
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
use sdformat::{
    ErrorCode, ForceTorque, ForceTorqueFrame as Frame, ForceTorqueMeasureDirection as Direction,
    NoResources, NoiseKind, SensorConfig, load_str,
};
fn xml(sensor: &str) -> String {
    format!(
        "<sdf version='1.12'><model name='m'><link name='p'><pose>1 0 0 0 0 0</pose></link><link name='c'><pose>2 0 0 0 0 0</pose></link><joint name='j' type='fixed'><pose>0 3 0 0 0 0</pose><parent>p</parent><child>c</child>{sensor}</joint></model></sdf>"
    )
}
fn sensor(body: &str) -> sdformat::Sensor {
    let text = xml(&format!(
        "<sensor name='s' type='force_torque'>{body}</sensor>"
    ));
    let mut sdf = load_str(&text, "joint-sensors.sdf", &NoResources).unwrap();
    sdf.models.remove(0).joints.remove(0).sensors.remove(0)
}
fn pose(actual: sdformat::Transform, expected: [f64; 3]) {
    for (a, b) in actual.translation.into_iter().zip(expected) {
        assert!((a - b).abs() <= 1e-3);
    }
    for (a, b) in actual.rotation.into_iter().zip([0., 0., 0., 1.]) {
        assert!((a - b).abs() <= 1e-3);
    }
}
fn double(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite());
    let ordered = |v: f64| {
        let bits = v.to_bits();
        if bits & (1 << 63) != 0 {
            (!bits).wrapping_add(1)
        } else {
            bits | (1 << 63)
        }
    };
    assert!(
        ordered(actual).abs_diff(ordered(expected)) <= 4,
        "{actual} versus {expected}"
    );
}
#[test]
fn original_joint_sensors_complete_assertions() {
    let result = load_str(
        include_str!("fixtures/joint_sensors.sdf"),
        "joint_sensors.sdf",
        &NoResources,
    );
    assert!(result.is_ok());
    let sdf = result.unwrap();
    let model = sdf.models.first();
    assert!(model.is_some());
    let model = model.unwrap();
    assert_eq!(model.name, "model");
    let joint = model.joints.first();
    assert!(joint.is_some());
    let joint = joint.unwrap();
    assert_eq!(joint.name, "joint");
    assert_eq!(joint.sensors.len(), 1);
    let sensor = joint
        .sensors
        .iter()
        .find(|s| s.name == "force_torque_sensor");
    assert!(sensor.is_some());
    let sensor = sensor.unwrap();
    assert_eq!(sensor.name, "force_torque_sensor");
    assert_eq!(sensor.kind, "force_torque");
    assert!(matches!(sensor.config, SensorConfig::ForceTorque(_)));
    pose(sensor.pose.transform, [10., 11., 12.]);
    let joint_frame = format!("{}::{}", model.scoped_name, joint.name);
    let resolved = sdf.resolve_pose(
        &sensor.pose,
        &model.scoped_name,
        &joint_frame,
        &model.scoped_name,
    );
    assert!(resolved.is_ok());
    pose(resolved.unwrap(), [10., 11., 12.]);
    let resolved = sdf.resolve_pose(&sensor.pose, &model.scoped_name, &joint_frame, &joint_frame);
    assert!(resolved.is_ok());
    pose(resolved.unwrap(), [10., 11., 12.]);
    let config = sensor.force_torque();
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.frame, Frame::Parent);
    assert_eq!(config.measure_direction, Direction::ParentToChild);
    for (index, noise) in config.force.iter().chain(&config.torque).enumerate() {
        double(noise.mean, index as f64);
        double(noise.stddev, index as f64 + 0.1);
    }
}
#[test]
fn force_torque_defaults_preserve_selected_variant_and_explicit_empty_groups() {
    let absent = sensor("");
    assert!(absent.imu().is_none() && absent.contact().is_none());
    assert_eq!(absent.force_torque().unwrap(), &ForceTorque::default());
    assert!(absent.configuration_xml.is_empty());
    let explicit =
        sensor("<force_torque><frame/><measure_direction/><force><x/></force></force_torque>");
    let cfg = explicit.force_torque().unwrap();
    assert_eq!(cfg.frame, Frame::Child);
    assert_eq!(cfg.measure_direction, Direction::ChildToParent);
    assert_eq!(cfg.force[0].kind, NoiseKind::None);
    assert_eq!(cfg.source.as_ref().unwrap().uri, "joint-sensors.sdf");
    assert!(explicit.configuration_xml.starts_with("<force_torque>"));
    assert_eq!(sensor("<update_rate>-1</update_rate>").update_rate, -1.);
}
#[test]
fn joint_sensor_default_frame_differs_from_child_link_and_errors_remain_deferred() {
    for (reference, model_position, joint_position) in [
        ("", [2., 3., 4.], [0., 0., 4.]),
        (" relative_to='__model__'", [0., 0., 4.], [-2., -3., 4.]),
        (" relative_to='p'", [1., 0., 4.], [-1., -3., 4.]),
    ] {
        let text = xml(&format!(
            "<sensor name='s' type='force_torque'><pose{reference}>0 0 4 0 0 0</pose></sensor>"
        ));
        let sdf = load_str(&text, "frames", &NoResources).unwrap();
        let model = &sdf.models[0];
        let sensor = &model.joints[0].sensors[0];
        pose(
            sdf.resolve_pose(&sensor.pose, &model.scoped_name, "m::j", &model.scoped_name)
                .unwrap(),
            model_position,
        );
        pose(
            sdf.resolve_pose(&sensor.pose, &model.scoped_name, "m::j", "m::j")
                .unwrap(),
            joint_position,
        );
        assert!(sdf.validate_non_frame_poses().is_ok());
    }
    let text = xml("<sensor name='s' type='force_torque'><pose relative_to='missing'/></sensor>");
    let sdf = load_str(&text, "deferred", &NoResources).unwrap();
    assert_eq!(
        sdf.validate_non_frame_poses().unwrap_err().code,
        ErrorCode::UnknownReference
    );
}
#[test]
fn owned_joint_sensor_configuration_and_raw_xml_survive_document_drop_and_clone() {
    let raw = "<force_torque><frame><![CDATA[ sensor ]]></frame><measure_direction> parent_to_child </measure_direction><force><x><noise type=' gaussian '><mean>1.2</mean></noise></x></force></force_torque>";
    let mut original = sensor(raw);
    assert_eq!(original.configuration_xml, raw);
    assert_eq!(original.force_torque().unwrap().frame, Frame::Sensor);
    let mut copy = original.clone();
    let SensorConfig::ForceTorque(cfg) = &mut copy.config else {
        panic!("force/torque expected")
    };
    cfg.force[0].mean = 9.;
    cfg.source.as_mut().unwrap().uri.clear();
    copy.configuration_xml.clear();
    assert_eq!(original.force_torque().unwrap().force[0].mean, 1.2);
    assert_eq!(original.configuration_xml, raw);
    assert_eq!(
        original
            .force_torque()
            .unwrap()
            .source
            .as_ref()
            .unwrap()
            .uri,
        "joint-sensors.sdf"
    );
    let SensorConfig::ForceTorque(cfg) = &mut original.config else {
        panic!("force/torque expected")
    };
    cfg.force[1].mean = 3.;
    assert_eq!(copy.force_torque().unwrap().force[1].mean, 0.);
}
#[test]
fn invalid_configurations_and_duplicate_joint_sensors_are_explicit_errors() {
    for (body, code) in [
        (
            "<force_torque><frame>PARENT</frame></force_torque>",
            ErrorCode::InvalidValue,
        ),
        (
            "<force_torque><measure_direction>BAD</measure_direction></force_torque>",
            ErrorCode::InvalidValue,
        ),
        (
            "<force_torque><force><x><noise/></x></force></force_torque>",
            ErrorCode::Missing,
        ),
        (
            "<force_torque><force><w/></force></force_torque>",
            ErrorCode::Unsupported,
        ),
        ("<imu/>", ErrorCode::Unsupported),
        ("<contact/>", ErrorCode::Unsupported),
    ] {
        let text = xml(&format!(
            "<sensor name='s' type='force_torque'>{body}</sensor>"
        ));
        assert_eq!(
            load_str(&text, "invalid", &NoResources).unwrap_err().code,
            code,
            "{body}"
        );
    }
    let text = xml("<sensor name='s' type='force_torque'/><sensor name='s' type='force_torque'/>");
    assert_eq!(
        load_str(&text, "duplicate", &NoResources).unwrap_err().code,
        ErrorCode::DuplicateName
    );
}

fn unhex(value: &str) -> String {
    if value == "-" {
        return String::new();
    }
    String::from_utf8(
        (0..value.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
            .collect(),
    )
    .unwrap()
}
fn native_pose<'a>(expected: &mut impl Iterator<Item = &'a str>) -> [f64; 7] {
    std::array::from_fn(|_| f64::from_bits(expected.next().unwrap().parse().unwrap()))
}
fn compare_pose(actual: sdformat::Transform, expected: [f64; 7], id: &str) {
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected)
    {
        assert!((a - b).abs() <= 1e-12, "{id}: {a} versus {b}");
    }
}
#[test]
fn actual_native_force_torque_configuration_and_pose_corpus() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("force_torque.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 96);
    for row in cases.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let text = row["xml"].as_str().unwrap();
        let tokens = row["tokens"].as_array().unwrap();
        struct Import<'a>(Option<&'a str>);
        impl sdformat::ResourceResolver for Import<'_> {
            fn resolve(&self, uri: &str, parent: &str) -> Result<sdformat::Resource, String> {
                assert_eq!(uri, "resource.sdf");
                assert_eq!(parent, "native-force-torque.sdf");
                Ok(sdformat::Resource {
                    canonical_uri: "memory://imported.sdf".into(),
                    xml: self.0.expect("explicit corpus resource").into(),
                })
            }
        }
        let imported = row["resource"].as_str();
        let source_xml = imported.unwrap_or(text);
        let parsed = load_str(text, "native-force-torque.sdf", &Import(imported));
        if row.get("gap").is_some() {
            assert!(matches!(
                id,
                "unknown_axis"
                    | "unknown_attribute"
                    | "namespaced_frame"
                    | "unused_imu"
                    | "unused_contact"
                    | "duplicate_group"
                    | "duplicate_direction"
                    | "rate_inf"
            ));
            assert_eq!(tokens[0], "0", "{id}");
        }
        if tokens[0] != "0" || row.get("gap").is_some() {
            let expected = match row["rust_error"].as_str().unwrap() {
                "Missing" => ErrorCode::Missing,
                "InvalidValue" => ErrorCode::InvalidValue,
                "Unsupported" => ErrorCode::Unsupported,
                "DuplicateName" => ErrorCode::DuplicateName,
                other => panic!("unqualified error {other}: {id}"),
            };
            assert_eq!(
                parsed
                    .err()
                    .unwrap_or_else(|| panic!("{id}: unexpectedly accepted"))
                    .code,
                expected,
                "{id}"
            );
            continue;
        }
        let sdf = parsed.unwrap_or_else(|error| panic!("{id}: {error}"));
        let root = &sdf.models[0];
        let model = root.models.first().unwrap_or(root);
        let (sensors, owner) = if row["owner"] == "joint" {
            (
                &model.joints[0].sensors,
                format!("{}::{}", model.scoped_name, model.joints[0].name),
            )
        } else {
            (&model.links[0].sensors, model.links[0].scoped_name.clone())
        };
        let sensor = &sensors[0];
        if imported.is_some() {
            assert_eq!(sensor.source.uri, "memory://imported.sdf", "{id}");
            let raw = source_xml
                .find("<force_torque")
                .map(|begin| {
                    let end = source_xml[begin..].find("</force_torque>").unwrap()
                        + begin
                        + "</force_torque>".len();
                    &source_xml[begin..end]
                })
                .unwrap_or("");
            assert_eq!(sensor.configuration_xml, raw, "{id}");
            if let Some(source) = &sensor.force_torque().unwrap().source {
                assert_eq!(source.uri, "memory://imported.sdf", "{id}");
            }
        }
        let mut expected = tokens.iter().skip(1).map(|v| v.as_str().unwrap());
        assert_eq!(sensor.kind, unhex(expected.next().unwrap()), "{id}");
        assert_eq!(
            sensor.force_torque().is_some(),
            expected.next().unwrap() == "1",
            "{id}"
        );
        assert_eq!(
            sensor.imu().is_some(),
            expected.next().unwrap() == "1",
            "{id}"
        );
        assert_eq!(
            sensors.len(),
            expected.next().unwrap().parse::<usize>().unwrap(),
            "{id}"
        );
        if let Some(cfg) = sensor.force_torque() {
            let frame = match cfg.frame {
                Frame::Parent => 1,
                Frame::Child => 2,
                Frame::Sensor => 3,
            };
            assert_eq!(
                frame,
                expected.next().unwrap().parse::<u8>().unwrap(),
                "{id}"
            );
            let direction = match cfg.measure_direction {
                Direction::ParentToChild => 1,
                Direction::ChildToParent => 2,
            };
            assert_eq!(
                direction,
                expected.next().unwrap().parse::<u8>().unwrap(),
                "{id}"
            );
            for noise in cfg.force.iter().chain(&cfg.torque) {
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
            assert_eq!(
                cfg.source.is_some(),
                source_xml.contains("<force_torque"),
                "{id}"
            );
            assert_eq!(
                sensor.configuration_xml.is_empty(),
                !source_xml.contains("<force_torque"),
                "{id}"
            );
        }
        assert_eq!(
            sensor.update_rate.to_bits(),
            expected.next().unwrap().parse::<u64>().unwrap(),
            "{id}"
        );
        assert_eq!(sensor.always_on, expected.next().unwrap() == "1", "{id}");
        assert_eq!(
            sensor.enable_metrics,
            expected.next().unwrap() == "1",
            "{id}"
        );
        assert_eq!(sensor.topic, unhex(expected.next().unwrap()), "{id}");
        assert_eq!(sensor.frame_id, unhex(expected.next().unwrap()), "{id}");
        compare_pose(sensor.pose.transform, native_pose(&mut expected), id);
        let mut deferred = false;
        for target in [&model.scoped_name, &owner] {
            let count = expected.next().unwrap().parse::<usize>().unwrap();
            let actual = sdf.resolve_pose(&sensor.pose, &model.scoped_name, &owner, target);
            if count == 0 {
                compare_pose(actual.unwrap(), native_pose(&mut expected), id);
            } else {
                for _ in 0..count {
                    assert_eq!(expected.next(), Some("26"), "{id}");
                }
                assert_eq!(
                    actual.unwrap_err().code,
                    ErrorCode::UnknownReference,
                    "{id}"
                );
                deferred = true;
            }
        }
        assert_eq!(sdf.validate_non_frame_poses().is_err(), deferred, "{id}");
        assert!(expected.next().is_none(), "{id}");
    }
}

#[test]
fn nested_world_joint_sensors_validate_and_preserve_owner_local_name_uniqueness() {
    let nested = xml("<sensor name='s' type='force_torque'/><sensor name='second' type='imu'/>");
    let model = nested
        .strip_prefix("<sdf version='1.12'>")
        .unwrap()
        .strip_suffix("</sdf>")
        .unwrap();
    let text = format!(
        "<sdf version='1.12'><world name='w'><model name='outer'>{model}</model></world></sdf>"
    );
    let sdf = load_str(&text, "world.sdf", &NoResources).unwrap();
    let model = &sdf.worlds[0].models[0].models[0];
    assert_eq!(model.scoped_name, "w::outer::m");
    let sensors = &model.joints[0].sensors;
    assert_eq!(
        sensors.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        ["s", "second"]
    );
    assert!(sensors[1].imu().is_some());
    assert!(sensors[1].force_torque().is_none());
    assert!(sensors[0].contact().is_none());
    assert!(sdf.validate_non_frame_poses().is_ok());
    pose(
        sdf.resolve_pose(&sensors[0].pose, &model.scoped_name, "w::outer::m::j", "w")
            .unwrap(),
        [2., 3., 0.],
    );
    let bad = text.replace(
        "<sensor name='second' type='imu'/>",
        "<sensor name='second' type='imu'><pose relative_to='missing'/></sensor>",
    );
    let loaded = load_str(&bad, "world.sdf", &NoResources).unwrap();
    let error = loaded.validate_non_frame_poses().unwrap_err();
    assert_eq!(error.code, ErrorCode::UnknownReference);
    assert_eq!(error.source.uri, "world.sdf");
    // Names belong to each owner, rather than a global sensor namespace.
    let text = xml("<sensor name='s' type='force_torque'/>").replace(
        "<link name='p'>",
        "<link name='p'><sensor name='s' type='contact'/>",
    );
    let loaded = load_str(&text, "owners.sdf", &NoResources).unwrap();
    assert_eq!(
        loaded.models[0].links[0].sensors[0].name,
        loaded.models[0].joints[0].sensors[0].name
    );
}
