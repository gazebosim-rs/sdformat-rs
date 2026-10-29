// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Contact metadata output; collision references remain unresolved.
use crate::{ContactSensor, Diagnostic, ErrorCode, Source};
use quick_xml::escape::escape;

fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<contact>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
pub(crate) fn validate_config(value: &ContactSensor) -> Result<(), Diagnostic> {
    if value.collisions.is_empty() {
        return Err(error(
            ErrorCode::InvalidValue,
            "empty contact collision lists cannot roundtrip",
        ));
    }
    for text in value.collisions.iter().chain(std::iter::once(&value.topic)) {
        if text.is_empty()
            || text.trim() != text
            || text
                .chars()
                .any(|c| c < '\u{20}' || matches!(c, '\u{fffe}' | '\u{ffff}'))
        {
            return Err(error(
                ErrorCode::InvalidValue,
                "contact text cannot roundtrip through XML",
            ));
        }
        // Native Param reads only the first chunk of split CDATA.
        if text.contains("  ") && text.contains("]]>") {
            return Err(error(
                ErrorCode::Unsupported,
                "contact text requires CDATA that native loading would truncate",
            ));
        }
    }
    Ok(())
}
fn text(value: &str) -> String {
    if value.contains("  ") {
        format!("<![CDATA[{value}]]>")
    } else {
        escape(value).into_owned()
    }
}
impl ContactSensor {
    /// Emit ordered collision strings and topic without resolving names or contacts.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        let mut output = String::from("<contact>");
        for collision in &self.collisions {
            output.push_str(&format!("<collision>{}</collision>", text(collision)));
        }
        output.push_str(&format!("<topic>{}</topic></contact>", text(&self.topic)));
        Ok(output)
    }
}
