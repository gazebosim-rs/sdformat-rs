// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Collision metadata serialization without physics or resource access.
use crate::{Collision, Diagnostic, ErrorCode, Source};
use quick_xml::escape::escape;

fn error(message: &str) -> Diagnostic {
    Diagnostic {
        code: ErrorCode::InvalidValue,
        source: Source {
            uri: "<collision>".into(),
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
impl Collision {
    /// Emit supported metadata; unrepresentable values return an error.
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
            return Err(error("collision name or pose frame cannot roundtrip"));
        }
        self.pose
            .transform
            .validate(&error("invalid pose").source)?;
        let p = self.pose.transform.translation;
        let q = self.pose.transform.rotation;
        if p.iter().chain(&q).any(|v| v.is_subnormal()) {
            return Err(error("subnormal pose cannot roundtrip"));
        }
        let density = match self.density {
            Some(value) if !value.is_finite() || value.is_subnormal() => {
                return Err(error("density cannot roundtrip"));
            }
            Some(value) => format!("<density>{value}</density>"),
            None => String::new(),
        };
        let geometry = self.geometry.to_xml()?;
        let surface = self.surface.to_xml()?;
        Ok(format!(
            "<collision name=\"{}\"><pose relative_to=\"{}\" rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose>{density}{geometry}{surface}</collision>",
            escape(name),
            escape(&self.pose.relative_to),
            p[0],
            p[1],
            p[2],
            q[0],
            q[1],
            q[2],
            q[3]
        ))
    }
}
