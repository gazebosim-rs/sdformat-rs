// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Bounded typed sensor output; retained payloads are never silently dropped.
use crate::{Diagnostic, ErrorCode, Sensor, SensorConfig, Source};
use quick_xml::escape::escape;
fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<sensor>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn attribute(value: &str) -> bool {
    value.trim() == value
        && value
            .chars()
            .all(|c| c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
}
fn text(value: &str) -> bool {
    attribute(value) && !value.contains("  ")
}
pub(crate) fn validate(sensor: &Sensor) -> Result<(), Diagnostic> {
    if !matches!(
        (sensor.kind.as_str(), &sensor.config),
        ("force_torque", SensorConfig::ForceTorque(_))
            | ("imu", SensorConfig::Imu(_))
            | ("altimeter", SensorConfig::Altimeter(_))
    ) || !sensor.plugins.is_empty()
        || !sensor.extensions.is_empty()
        || sensor.gz_frame_id.is_some()
    {
        return Err(error(
            ErrorCode::Unsupported,
            "sensor XML cannot preserve this family, plugin or extension payload",
        ));
    }
    let name = &sensor.name;
    if name.is_empty()
        || !attribute(name)
        || name.contains("::")
        || name == "world"
        || (name.len() >= 4 && name.starts_with("__") && name.ends_with("__"))
        || !attribute(&sensor.pose.relative_to)
        || !text(&sensor.frame_id)
        || !text(&sensor.topic)
        || sensor.topic == "__default__"
        || !sensor.update_rate.is_finite()
        || sensor.update_rate.is_subnormal()
        || (sensor.kind == "imu" && sensor.update_rate < 0.)
    {
        return Err(error(
            ErrorCode::InvalidValue,
            "sensor envelope cannot roundtrip through XML",
        ));
    }
    sensor.pose.transform.validate(&Source {
        uri: "<sensor>".into(),
        line: 0,
        column: 0,
    })?;
    if sensor
        .pose
        .transform
        .translation
        .iter()
        .chain(&sensor.pose.transform.rotation)
        .any(|n| n.is_subnormal())
    {
        return Err(error(
            ErrorCode::InvalidValue,
            "sensor pose cannot roundtrip through XML",
        ));
    }
    match &sensor.config {
        SensorConfig::ForceTorque(config) => crate::force_torque_xml::validate_config(config),
        SensorConfig::Imu(config) => crate::imu_xml::validate_config(config),
        SensorConfig::Altimeter(config) => crate::altimeter_xml::validate_config(config),
        _ => Err(error(
            ErrorCode::Unsupported,
            "unsupported sensor configuration",
        )),
    }
}
impl Sensor {
    /// Emit a supported IMU, force/torque or altimeter sensor with current values.
    /// Other families, plugins and opaque extensions return Unsupported.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate(self)?;
        let config = match &self.config {
            SensorConfig::ForceTorque(value) => value.to_xml()?,
            SensorConfig::Imu(value) => value.to_xml()?,
            SensorConfig::Altimeter(value) => value.to_xml()?,
            _ => {
                return Err(error(
                    ErrorCode::Unsupported,
                    "unsupported sensor configuration",
                ));
            }
        };
        let p = self.pose.transform.translation;
        let q = self.pose.transform.rotation;
        Ok(format!(
            "<sensor name=\"{}\" type=\"{}\"><pose relative_to=\"{}\" rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose><frame_id>{}</frame_id><topic>{}</topic><update_rate>{}</update_rate><always_on>{}</always_on><visualize>{}</visualize><enable_metrics>{}</enable_metrics>{config}</sensor>",
            escape(&self.name),
            self.kind,
            escape(&self.pose.relative_to),
            p[0],
            p[1],
            p[2],
            q[0],
            q[1],
            q[2],
            q[3],
            escape(&self.frame_id),
            escape(&self.topic),
            self.update_rate,
            self.always_on,
            self.visualize,
            self.enable_metrics
        ))
    }
}
