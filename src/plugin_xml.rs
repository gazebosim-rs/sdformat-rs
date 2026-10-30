// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Owned opaque Plugin metadata; no shared library or simulation system is loaded.
use crate::{Diagnostic, ErrorCode, Plugin, Source};
use quick_xml::escape::escape;
fn attribute(value: &str) -> bool {
    value.trim_matches([' ', '\t', '\n']) == value
        && value
            .chars()
            .all(|c| c >= '\u{20}' && !matches!(c, '\u{fffe}' | '\u{ffff}'))
}
pub(crate) fn serialize(plugin: &Plugin, scope: &str) -> Result<String, Diagnostic> {
    let error = |code: ErrorCode, message: &str| Diagnostic {
        code,
        source: Source {
            uri: scope.into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    };
    if plugin.filename.is_empty() || !attribute(&plugin.name) || !attribute(&plugin.filename) {
        return Err(error(
            ErrorCode::InvalidValue,
            "plugin attributes cannot roundtrip",
        ));
    }
    if plugin.xml.is_empty() {
        return Ok(format!(
            "<plugin name=\"{}\" filename=\"{}\"/>",
            escape(&plugin.name),
            escape(&plugin.filename)
        ));
    }
    let doc = crate::xml::Document::parse(&plugin.xml)
        .map_err(|_| error(ErrorCode::Xml, "plugin payload is not one XML element"))?;
    let root = doc.root_element();
    let trim = |value: &str| value.trim_matches([' ', '\t', '\n']).to_owned();
    if root.range() != (0..plugin.xml.len())
        || !root.has_tag_name("plugin")
        || trim(root.attribute("name").unwrap_or("")) != plugin.name
        || root.attribute("filename").map(trim).as_deref() != Some(plugin.filename.as_str())
    {
        return Err(error(
            ErrorCode::Unsupported,
            "plugin raw XML disagrees with owned attributes",
        ));
    }
    Ok(plugin.xml.clone())
}
impl Plugin {
    /// Emit opaque metadata with explicit owned-attribute consistency checks.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        serialize(self, "<plugin>")
    }
}
