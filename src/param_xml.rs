// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! One native Param text chunk; callers reject unrepresentable values first.
use quick_xml::escape::escape;
pub(crate) fn needs_cdata(value: &str) -> bool {
    value.contains("  ") || value.contains(['\t', '\n'])
}
pub(crate) fn text(value: &str) -> String {
    if needs_cdata(value) {
        format!("<![CDATA[{value}]]>")
    } else {
        escape(value).into_owned()
    }
}

/// Validate strings that native Param must retain as one complete text chunk.
pub(crate) fn validate_text(value: &str) -> Result<(), crate::ErrorCode> {
    if value.is_empty()
        || value.trim_matches([' ', '\t', '\n']) != value
        || value.chars().any(|c| {
            (c < '\u{20}' && c != '\t' && c != '\n') || matches!(c, '\u{fffe}' | '\u{ffff}')
        })
    {
        return Err(crate::ErrorCode::InvalidValue);
    }
    if needs_cdata(value) && value.contains("]]>") {
        return Err(crate::ErrorCode::Unsupported);
    }
    Ok(())
}
