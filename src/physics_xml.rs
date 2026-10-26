// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

//! Bounded XML output for owned world physics profile metadata.
use crate::{Diagnostic, ErrorCode, Physics, Source};
use quick_xml::escape::escape;

fn roundtrip_attribute(value: &str) -> bool {
    // The profile reader trims native attribute padding; XML also normalizes
    // control whitespace. Neither change may silently alter an emitted value.
    !value.is_empty()
        && value.trim_matches([' ', '\t', '\n']) == value
        && value
            .chars()
            .all(|c| c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
}

impl Physics {
    /// Serialize supported profile metadata as one SDF 1.12 `<physics>` element.
    /// This does not configure or select a physics engine.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let reserved_name = self.name == "world"
            || (self.name.len() >= 4 && self.name.starts_with("__") && self.name.ends_with("__"));
        if !roundtrip_attribute(&self.name)
            || !roundtrip_attribute(&self.engine_type)
            || reserved_name
            || self.name.contains("::")
            || [
                self.max_step_size,
                self.real_time_factor,
                self.real_time_update_rate,
            ]
            .iter()
            .any(|v| !v.is_finite() || v.is_subnormal())
        {
            return Err(Diagnostic {
                code: ErrorCode::InvalidValue,
                source: Source {
                    uri: "<physics>".into(),
                    line: 0,
                    column: 0,
                },
                message: "physics profile values cannot roundtrip through bounded XML".into(),
            });
        }
        Ok(format!(
            "<physics name=\"{}\" type=\"{}\" default=\"{}\">\
             <max_step_size>{}</max_step_size><real_time_factor>{}</real_time_factor>\
             <real_time_update_rate>{}</real_time_update_rate><max_contacts>{}</max_contacts></physics>",
            escape(&self.name),
            escape(&self.engine_type),
            self.is_default,
            self.max_step_size,
            self.real_time_factor,
            self.real_time_update_rate,
            self.max_contacts,
        ))
    }
}
