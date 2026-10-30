// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Primitive geometry metadata; consumers choose physical admission separately.
use crate::{Diagnostic, ErrorCode, Geometry, Source};
fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<geometry>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn dimension(name: &str, value: f64) -> Result<String, Diagnostic> {
    if !value.is_finite() || value <= 0. || value.is_subnormal() {
        return Err(error(
            ErrorCode::InvalidValue,
            "dimension cannot roundtrip through the supported XML reader",
        ));
    }
    Ok(format!("<{name}>{value}</{name}>"))
}
fn vector<const N: usize>(
    name: &str,
    value: [f64; N],
    positive: bool,
) -> Result<String, Diagnostic> {
    if value
        .iter()
        .any(|v| !v.is_finite() || (positive && *v <= 0.))
    {
        return Err(error(ErrorCode::InvalidValue, "invalid geometry vector"));
    }
    let text = value.map(|v| v.to_string()).join(" ");
    Ok(format!("<{name}>{text}</{name}>"))
}
impl Geometry {
    /// Emit supported geometry metadata. Plane normal normalization occurs on
    /// reload under the existing reader policy; emission does not change self.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let shape = match self {
            Self::Empty => "<empty/>".into(),
            Self::Box { size } => format!("<box>{}</box>", vector("size", *size, true)?),
            Self::Sphere { radius } => {
                format!("<sphere>{}</sphere>", dimension("radius", *radius)?)
            }
            Self::Cylinder { radius, length }
            | Self::Capsule { radius, length }
            | Self::Cone { radius, length } => {
                let name = match self {
                    Self::Cylinder { .. } => "cylinder",
                    Self::Capsule { .. } => "capsule",
                    _ => "cone",
                };
                format!(
                    "<{name}>{}{}</{name}>",
                    dimension("radius", *radius)?,
                    dimension("length", *length)?
                )
            }
            Self::Ellipsoid { radii } => {
                format!("<ellipsoid>{}</ellipsoid>", vector("radii", *radii, true)?)
            }
            Self::Plane { normal, size } => format!(
                "<plane>{}{}</plane>",
                vector("normal", *normal, false)?,
                vector("size", *size, true)?
            ),
            Self::Mesh(value) => value.to_xml()?,
            Self::Heightmap(value) => value.to_xml()?,
            Self::Polyline(lines) => {
                if lines.is_empty() {
                    return Err(error(
                        ErrorCode::Unsupported,
                        "empty polyline groups cannot roundtrip through XML",
                    ));
                }
                let mut output = String::new();
                for line in lines {
                    output.push_str(&line.to_xml()?);
                }
                output
            }
        };
        Ok(format!("<geometry>{shape}</geometry>"))
    }
}
