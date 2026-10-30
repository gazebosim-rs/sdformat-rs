// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Sea-level atmosphere metadata; no altitude or weather model is evaluated.
use crate::{Atmosphere, AtmosphereType, Diagnostic, ErrorCode, Source};
impl Atmosphere {
    /// Emit current typed values; retained source/XML describe input origin.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        if [
            self.temperature_kelvin,
            self.temperature_gradient,
            self.pressure,
        ]
        .iter()
        .any(|v| !v.is_finite() || v.is_subnormal())
        {
            return Err(Diagnostic {
                code: ErrorCode::InvalidValue,
                source: Source {
                    uri: "<atmosphere>".into(),
                    line: 0,
                    column: 0,
                },
                message: "atmosphere scalars cannot roundtrip through XML".into(),
            });
        }
        let kind = match self.atmosphere_type {
            AtmosphereType::Adiabatic => "adiabatic",
        };
        Ok(format!(
            "<atmosphere type=\"{kind}\"><temperature>{}</temperature><temperature_gradient>{}</temperature_gradient><pressure>{}</pressure></atmosphere>",
            self.temperature_kelvin, self.temperature_gradient, self.pressure
        ))
    }
}
