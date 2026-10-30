// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Visual metadata; no geometry asset, plugin library or renderer is loaded.
use crate::{Diagnostic, ErrorCode, Source, Visual};
use quick_xml::escape::escape;
fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<visual>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn attribute(value: &str) -> bool {
    value.trim_matches([' ', '\t', '\n']) == value
        && value
            .chars()
            .all(|c| c >= '\u{20}' && !matches!(c, '\u{fffe}' | '\u{ffff}'))
}

impl Visual {
    /// Emit current typed metadata, retaining opaque plugin XML unchanged.
    /// Empty geometry is the existing `None` state; no resource is read.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let name = &self.name;
        if name.is_empty()
            || name.trim() != name
            || !attribute(name)
            || name.contains("::")
            || name == "world"
            || (name.len() >= 4 && name.starts_with("__") && name.ends_with("__"))
            || !attribute(&self.pose.relative_to)
        {
            return Err(error(
                ErrorCode::InvalidValue,
                "visual name or pose frame cannot roundtrip",
            ));
        }
        self.pose
            .transform
            .validate(&error(ErrorCode::InvalidValue, "invalid pose").source)?;
        let p = self.pose.transform.translation;
        let q = self.pose.transform.rotation;
        if p.iter().chain(&q).any(|v| v.is_subnormal()) {
            return Err(error(
                ErrorCode::InvalidValue,
                "subnormal pose cannot roundtrip",
            ));
        }
        let transparency = f64::from(self.properties.transparency);
        if !transparency.is_finite()
            || !crate::number::stored_double_to_float(transparency)
                .is_ok_and(|value| value.to_bits() == self.properties.transparency.to_bits())
        {
            return Err(error(
                ErrorCode::InvalidValue,
                "transparency cannot roundtrip",
            ));
        }
        let retro = match self.properties.laser_retro {
            Some(value) if !value.is_finite() || value.is_subnormal() => {
                return Err(error(
                    ErrorCode::InvalidValue,
                    "laser retro cannot roundtrip",
                ));
            }
            Some(value) => format!("<laser_retro>{value}</laser_retro>"),
            None => String::new(),
        };
        let geometry = self
            .geometry
            .as_ref()
            .map_or_else(|| Ok("<geometry/>".into()), |g| g.to_xml())?;
        let material = self
            .material
            .as_ref()
            .map_or_else(|| Ok(String::new()), |m| m.to_xml())?;
        let plugins = self
            .plugins
            .iter()
            .map(|plugin| crate::plugin_xml::serialize(plugin, "<visual>"))
            .collect::<Result<Vec<_>, _>>()?
            .join("");
        Ok(format!(
            "<visual name=\"{}\"><pose relative_to=\"{}\" rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose>{geometry}<cast_shadows>{}</cast_shadows><transparency>{transparency}</transparency><visibility_flags>{}</visibility_flags>{retro}{material}{plugins}</visual>",
            escape(name),
            escape(&self.pose.relative_to),
            p[0],
            p[1],
            p[2],
            q[0],
            q[1],
            q[2],
            q[3],
            self.properties.cast_shadows,
            self.properties.visibility_flags
        ))
    }
}
