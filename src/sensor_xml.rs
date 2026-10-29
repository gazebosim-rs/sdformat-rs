// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Bounded force/torque sensor output; retained payloads are never silently dropped.
use crate::{Diagnostic, ErrorCode, ForceTorque, Sensor, SensorConfig, Source};
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
pub(crate) fn validate(sensor: &Sensor) -> Result<&ForceTorque, Diagnostic> {
    let SensorConfig::ForceTorque(config) = &sensor.config else {
        return Err(error(
            ErrorCode::Unsupported,
            "only force/torque sensor XML is supported",
        ));
    };
    if sensor.kind != "force_torque"
        || !sensor.plugins.is_empty()
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
    crate::force_torque_xml::validate_config(config)?;
    Ok(config)
}
impl Sensor {
    /// Emit a force/torque sensor with its current envelope and configuration.
    /// Other families, plugins and opaque extensions return Unsupported.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let config = validate(self)?.to_xml()?;
        let p = self.pose.transform.translation;
        let q = self.pose.transform.rotation;
        Ok(format!(
            "<sensor name=\"{}\" type=\"force_torque\"><pose relative_to=\"{}\" rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose><frame_id>{}</frame_id><topic>{}</topic><update_rate>{}</update_rate><always_on>{}</always_on><visualize>{}</visualize><enable_metrics>{}</enable_metrics>{config}</sensor>",
            escape(&self.name),
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
