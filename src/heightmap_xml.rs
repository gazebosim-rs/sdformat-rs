// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Heightmap metadata output; resource and terrain processing remain separate.
use crate::{Diagnostic, ErrorCode, Heightmap, Source};
fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<heightmap>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn text(value: &str) -> Result<String, Diagnostic> {
    crate::param_xml::validate_text(value)
        .map_err(|code| error(code, "heightmap text cannot roundtrip through native Param"))?;
    Ok(crate::param_xml::text(value))
}
fn scalar(name: &str, value: f64) -> Result<String, Diagnostic> {
    if !value.is_finite() || value.is_subnormal() {
        return Err(error(
            ErrorCode::InvalidValue,
            "heightmap scalar cannot roundtrip through the XML reader",
        ));
    }
    Ok(format!("<{name}>{value}</{name}>"))
}
fn vector(name: &str, value: [f64; 3]) -> Result<String, Diagnostic> {
    if value.iter().any(|v| !v.is_finite()) {
        return Err(error(
            ErrorCode::InvalidValue,
            "heightmap vector must be finite",
        ));
    }
    Ok(format!(
        "<{name}>{} {} {}</{name}>",
        value[0], value[1], value[2]
    ))
}
impl Heightmap {
    /// Emit ordered current metadata without image IO or terrain validation.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let mut xml = format!(
            "<heightmap><uri>{}</uri>{}{}<use_terrain_paging>{}</use_terrain_paging><sampling>{}</sampling>",
            text(&self.uri)?,
            vector("size", self.size)?,
            vector("pos", self.position)?,
            self.use_terrain_paging,
            self.sampling
        );
        for t in &self.textures {
            xml.push_str(&format!(
                "<texture>{}<diffuse>{}</diffuse><normal>{}</normal></texture>",
                scalar("size", t.size)?,
                text(&t.diffuse)?,
                text(&t.normal)?
            ));
        }
        for b in &self.blends {
            xml.push_str(&format!(
                "<blend>{}{}</blend>",
                scalar("min_height", b.min_height)?,
                scalar("fade_dist", b.fade_distance)?
            ));
        }
        xml.push_str("</heightmap>");
        Ok(xml)
    }
}
