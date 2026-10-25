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

use sdformat::{ErrorCode, NoResources, SensorConfig, SensorType, load_str};

fn fragment(fixture: &str, name: &str, kind: &str) -> String {
    let tag = format!("<sensor name=\"{name}\" type=\"{kind}\">");
    let start = fixture.find(&tag).unwrap();
    let end = start + fixture[start..].find("</sensor>").unwrap() + "</sensor>".len();
    format!(
        "<sdf version='1.10'><model name='model'><link name='link'>{}</link></model></sdf>",
        &fixture[start..end]
    )
}

#[test]
fn native_link_envelope_only_sensor_fragments_are_typed() {
    let fixture = include_str!("fixtures/sensors.sdf");
    for (name, kind, effective, position) in [
        (
            "logical_camera_sensor",
            "logical_camera",
            SensorType::LogicalCamera,
            [7., 8., 9.],
        ),
        (
            "multicamera_sensor",
            "multicamera",
            SensorType::Multicamera,
            [13., 14., 15.],
        ),
        ("rfid_sensor", "rfid", SensorType::Rfid, [4., 5., 6.]),
        ("rfid_tag", "rfidtag", SensorType::RfidTag, [7., 8., 9.]),
        ("sonar_sensor", "sonar", SensorType::Sonar, [10., 11., 12.]),
        (
            "wireless_receiver",
            "wireless_receiver",
            SensorType::WirelessReceiver,
            [13., 14., 15.],
        ),
        (
            "wireless_transmitter",
            "wireless_transmitter",
            SensorType::WirelessTransmitter,
            [1., 2., 3.],
        ),
    ] {
        let xml = fragment(fixture, name, kind);
        let sdf = load_str(&xml, "sensors.sdf#envelope-only", &NoResources).unwrap();
        let sensor = &sdf.models[0].links[0].sensors[0];
        assert_eq!(sensor.name, name);
        assert_eq!(sensor.kind, kind);
        assert_eq!(sensor.effective_type(), Some(effective));
        assert!(matches!(sensor.config, SensorConfig::Unconfigured));
        assert_eq!(sensor.pose.transform.translation, position);
        assert_eq!(sensor.pose.transform.rotation, [0., 0., 0., 1.]);
        assert!(!sensor.enable_metrics);
        assert!(sensor.configuration_xml.is_empty());
        assert!(sensor.imu().is_none());
    }
}

#[test]
fn native_known_envelope_only_type_still_rejects_unowned_payload() {
    let xml = "<sdf version='1.10'><model name='m'><link name='l'><sensor name='s' type='rfid'><imu/></sensor></link></model></sdf>";
    assert_eq!(
        load_str(xml, "unsupported-rfid-payload.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}
