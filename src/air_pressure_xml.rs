// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Pressure noise and reference-altitude metadata; no atmosphere is evaluated.
use crate::{AirPressure, Diagnostic, ErrorCode, Source};

pub(crate) fn validate_config(value: &AirPressure) -> Result<(), Diagnostic> {
    crate::noise_xml::validate(&value.pressure, "<air_pressure>")?;
    if !value.reference_altitude.is_finite() || value.reference_altitude.is_subnormal() {
        return Err(Diagnostic {
            code: ErrorCode::InvalidValue,
            source: Source {
                uri: "<air_pressure>".into(),
                line: 0,
                column: 0,
            },
            message: "reference altitude cannot roundtrip through XML".into(),
        });
    }
    Ok(())
}

impl AirPressure {
    /// Emit current noise and altitude without calculating pressure.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        Ok(format!(
            "<air_pressure><pressure>{}</pressure><reference_altitude>{}</reference_altitude></air_pressure>",
            crate::noise_xml::emit(&self.pressure),
            self.reference_altitude
        ))
    }
}
