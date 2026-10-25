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
// Partial original assertions: link_dom.cc DOMLink.Sensors contact subtree.
use sdformat::{ContactSensor, ErrorCode, NoResources, SensorConfig, load_str};
fn sensor(body: &str, kind: &str) -> sdformat::Sensor {
    let xml = format!(
        "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='{kind}'>{body}</sensor></link></model></sdf>"
    );
    let mut sdf = load_str(&xml, "contact.sdf", &NoResources).unwrap();
    sdf.models.remove(0).links.remove(0).sensors.remove(0)
}
#[test]
fn variants_have_no_dummy_configuration_and_preserve_effective_defaults() {
    let imu = sensor("", "imu");
    assert!(matches!(imu.config, SensorConfig::Imu(_)));
    assert!(imu.imu().is_some());
    assert!(imu.contact().is_none());
    let absent = sensor("", "contact");
    assert!(matches!(absent.config, SensorConfig::Contact(_)));
    assert!(absent.imu().is_none());
    assert_eq!(absent.contact().unwrap(), &ContactSensor::default());
    assert!(absent.configuration_xml.is_empty());
    let explicit = sensor("<contact/>", "contact");
    let contact = explicit.contact().unwrap();
    assert_eq!(contact.collisions, ["__default__"]);
    assert_eq!(contact.topic, "__default_topic__");
    assert_eq!(contact.source.as_ref().unwrap().uri, "contact.sdf");
    assert_eq!(explicit.configuration_xml, "<contact/>");
    let mut copied = imu.clone();
    let SensorConfig::Imu(config) = &mut copied.config else {
        panic!("IMU expected")
    };
    config.localization = "copy".into();
    assert_eq!(imu.imu().unwrap().localization, "CUSTOM");
}
#[test]
fn contact_strings_and_raw_configuration_are_owned_and_cloned() {
    let raw = "<contact>\n<collision><![CDATA[ arm  foot ]]></collision><collision>arm</collision><collision>arm</collision><topic> other  channel </topic></contact>";
    let original = sensor(&format!("<topic>envelope</topic>{raw}"), "contact");
    assert_eq!(original.topic, "envelope");
    let contact = original.contact().unwrap();
    assert_eq!(contact.collisions, ["arm  foot", "arm", "arm"]);
    assert_eq!(contact.topic, "other channel");
    assert_eq!(original.configuration_xml, raw);
    let mut clone = original.clone();
    let SensorConfig::Contact(c) = &mut clone.config else {
        panic!("contact expected")
    };
    c.collisions[0] = "copy".into();
    c.source.as_mut().unwrap().uri = "copy".into();
    clone.configuration_xml.clear();
    assert_eq!(original.contact().unwrap().collisions[0], "arm  foot");
    assert_eq!(
        original.contact().unwrap().source.as_ref().unwrap().uri,
        "contact.sdf"
    );
    assert_eq!(original.configuration_xml, raw);
}
#[test]
fn contact_references_and_poses_keep_their_distinct_validation_boundaries() {
    let xml = "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='contact'><pose relative_to='missing'/><contact><collision>missing::collision</collision></contact></sensor></link></model></sdf>";
    let sdf = load_str(xml, "deferred", &NoResources).unwrap();
    assert_eq!(
        sdf.models[0].links[0].sensors[0]
            .contact()
            .unwrap()
            .collisions,
        ["missing::collision"]
    );
    assert_eq!(
        sdf.validate_non_frame_poses().unwrap_err().code,
        ErrorCode::UnknownReference
    );
    assert_eq!(
        sensor("<update_rate>-1</update_rate>", "contact").update_rate,
        -1.
    );
}
#[test]
fn invalid_values_and_unknown_or_mismatched_payloads_do_not_default_to_imu() {
    for (kind, body, code) in [
        (
            "contact",
            "<contact><collision/></contact>",
            ErrorCode::Missing,
        ),
        ("contact", "<contact><topic/></contact>", ErrorCode::Missing),
        (
            "contact",
            "<contact><extra/></contact>",
            ErrorCode::Unsupported,
        ),
        (
            "contact",
            "<contact><ext:collision>c</ext:collision></contact>",
            ErrorCode::Unsupported,
        ),
        ("contact", "<imu/>", ErrorCode::Unsupported),
        ("imu", "<contact/>", ErrorCode::Unsupported),
        (
            "imu",
            "<update_rate>-1</update_rate>",
            ErrorCode::InvalidValue,
        ),
        ("camera", "", ErrorCode::Unsupported),
        ("none", "", ErrorCode::Unsupported),
        ("", "", ErrorCode::Missing),
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><sensor name='s' type='{kind}'>{body}</sensor></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "unsupported", &NoResources)
                .unwrap_err()
                .code,
            code,
            "{kind}: {body}"
        );
    }
}
fn pose(actual: sdformat::Transform, expected: [f64; 3]) {
    for (a, b) in actual.translation.into_iter().zip(expected) {
        assert!((a - b).abs() <= 1e-3);
    }
    for (a, b) in actual.rotation.into_iter().zip([0., 0., 0., 1.]) {
        assert!((a - b).abs() <= 1e-3);
    }
}
#[test]
fn original_link_dom_sensors_contact_observations() {
    let fixture = include_str!("fixtures/sensors.sdf");
    let start = fixture.find("<sensor name=\"contact_sensor\"").unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    let link_pose = "<pose> 0 0 3  0 0 0</pose>";
    assert!(fixture.contains(link_pose));
    let xml = format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{link_pose}{}</link></model></sdf>",
        &fixture[start..end]
    );
    let sdf = load_str(&xml, "sensors.sdf#contact_sensor", &NoResources).unwrap();
    let model = &sdf.models[0];
    let link = &model.links[0];
    let contact = link
        .sensors
        .iter()
        .find(|s| s.name == "contact_sensor")
        .unwrap();
    assert_eq!(contact.name, "contact_sensor");
    assert_eq!(contact.kind, "contact");
    pose(contact.pose.transform, [4., 5., 6.]);
    let resolved = sdf.resolve_pose(
        &contact.pose,
        &model.scoped_name,
        &link.scoped_name,
        &model.scoped_name,
    );
    assert!(resolved.is_ok());
    pose(resolved.unwrap(), [4., 5., 6.]);
    // Native default Resolve target is the owning link, with explicit model-relative raw pose.
    let resolved = sdf.resolve_pose(
        &contact.pose,
        &model.scoped_name,
        &link.scoped_name,
        &link.scoped_name,
    );
    assert!(resolved.is_ok());
    pose(resolved.unwrap(), [4., 5., 3.]);
    assert!(contact.enable_metrics);
}

fn unhex(value: &str) -> String {
    if value == "-" {
        return String::new();
    }
    let bytes = (0..value.len())
        .step_by(2)
        .map(|start| u8::from_str_radix(&value[start..start + 2], 16).unwrap())
        .collect();
    String::from_utf8(bytes).unwrap()
}
#[test]
fn native_contact_configuration_and_envelope_corpus() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("contact_sensor.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 85);
    for row in cases.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let tokens = row["tokens"].as_array().unwrap();
        let parsed = load_str(row["xml"].as_str().unwrap(), "corpus.sdf", &NoResources);
        if row.get("gap").is_some() {
            assert!(matches!(
                id,
                "imu_negative_rate_existing_policy"
                    | "unused_imu_payload"
                    | "unused_contact_payload"
                    | "unknown_extension_collision"
                    | "duplicate_topic"
                    | "duplicate_contact"
                    | "unknown_contact_attribute"
                    | "nonfinite_rate_inf"
                    | "nonfinite_rate_nan"
                    | "family_camera"
                    | "family_custom"
            ));
            assert_eq!(tokens[0], "0", "{id}");
        }
        if row.get("gap").is_some() || tokens[0] != "0" {
            let code = match row["rust_error"].as_str().unwrap() {
                "Missing" => ErrorCode::Missing,
                "InvalidValue" => ErrorCode::InvalidValue,
                "Unsupported" => ErrorCode::Unsupported,
                other => panic!("unqualified error {other}: {id}"),
            };
            assert_eq!(parsed.unwrap_err().code, code, "{id}");
            continue;
        }
        let sdf = parsed.unwrap_or_else(|error| panic!("{id}: {error}"));
        let sensor = &sdf.models[0].links[0].sensors[0];
        let mut expected = tokens.iter().skip(1).map(|v| v.as_str().unwrap());
        assert_eq!(sensor.kind, unhex(expected.next().unwrap()), "{id}");
        assert_eq!(
            sensor.imu().is_some(),
            expected.next().unwrap() == "1",
            "{id}"
        );
        let present = expected.next().unwrap() == "1";
        if let Some(contact) = sensor.contact() {
            assert_eq!(contact.source.is_some(), present, "{id}");
            assert_eq!(!sensor.configuration_xml.is_empty(), present, "{id}");
            let count: usize = expected.next().unwrap().parse().unwrap();
            let collisions: Vec<_> = expected.by_ref().take(count).map(unhex).collect();
            assert_eq!(contact.collisions, collisions, "{id}");
            assert_eq!(contact.topic, unhex(expected.next().unwrap()), "{id}");
        } else {
            assert!(!present, "{id}");
        }
        assert_eq!(sensor.topic, unhex(expected.next().unwrap()), "{id}");
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
        assert_eq!(sensor.frame_id, unhex(expected.next().unwrap()), "{id}");
        assert!(expected.next().is_none(), "{id}");
    }
}

#[test]
fn imu_envelope_errors_keep_exact_pre_enum_precedence() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("contact_sensor.json")).unwrap();
    let mut count = 0;
    for row in cases
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row.get("baseline_debug").is_some())
    {
        let error = load_str(row["xml"].as_str().unwrap(), "precedence", &NoResources).unwrap_err();
        assert_eq!(
            format!("{error:?}\n"),
            row["baseline_debug"].as_str().unwrap(),
            "{}",
            row["id"]
        );
        count += 1;
    }
    assert_eq!(count, 4);
}
