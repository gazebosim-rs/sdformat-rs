// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Ordered polyline metadata; polygon and collision admission remain separate.
use crate::{Diagnostic, ErrorCode, Polyline, Source};
fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<polyline>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
impl Polyline {
    /// Emit current height and points without inserting schema-default points.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        if self.points.is_empty() {
            return Err(error(
                ErrorCode::Unsupported,
                "empty point lists would gain a schema-default point on reload",
            ));
        }
        if !self.height.is_finite() || self.height.is_subnormal() {
            return Err(error(
                ErrorCode::InvalidValue,
                "polyline height cannot roundtrip through the XML reader",
            ));
        }
        if self.points.iter().flatten().any(|v| !v.is_finite()) {
            return Err(error(
                ErrorCode::InvalidValue,
                "polyline points must be finite",
            ));
        }
        let mut xml = format!("<polyline><height>{}</height>", self.height);
        for point in &self.points {
            xml.push_str(&format!("<point>{} {}</point>", point[0], point[1]));
        }
        xml.push_str("</polyline>");
        Ok(xml)
    }
}
