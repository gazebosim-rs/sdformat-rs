// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

//! Bounded standalone Frame XML output. Document-level graph rules are separate.
use crate::{Diagnostic, ErrorCode, Frame, Source};
use quick_xml::escape::escape;

fn safe_attribute(value: &str) -> bool {
    // XML normalizes attribute whitespace and forbids these code points.
    value
        .chars()
        .all(|c| c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
}

impl Frame {
    /// Serialize owned Frame values as one SDF 1.12 `<frame>` element.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let source = Source {
            uri: "<frame>".into(),
            line: 0,
            column: 0,
        };
        self.pose.transform.validate(&source)?;
        let reserved_name = self.name == "world"
            || (self.name.len() >= 4 && self.name.starts_with("__") && self.name.ends_with("__"));
        if self.name.is_empty()
            || self.name.trim() != self.name
            || self.name.contains("::")
            || reserved_name
            || !safe_attribute(&self.name)
            || !safe_attribute(&self.attached_to)
            || !safe_attribute(&self.pose.relative_to)
        {
            return Err(Diagnostic {
                code: ErrorCode::InvalidValue,
                source,
                message: "frame values cannot roundtrip through bounded XML".into(),
            });
        }
        let attached_to = if self.attached_to.is_empty() {
            String::new()
        } else {
            format!(" attached_to=\"{}\"", escape(&self.attached_to))
        };
        let relative_to = if self.pose.relative_to.is_empty() {
            String::new()
        } else {
            format!(" relative_to=\"{}\"", escape(&self.pose.relative_to))
        };
        let p = self.pose.transform.translation;
        let q = self.pose.transform.rotation;
        Ok(format!(
            "<frame name=\"{}\"{}><pose{} rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose></frame>",
            escape(&self.name),
            attached_to,
            relative_to,
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
