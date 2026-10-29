// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Pressure-noise metadata; air-speed measurement belongs to consumers.
use crate::{AirSpeed, Diagnostic};

pub(crate) fn validate_config(value: &AirSpeed) -> Result<(), Diagnostic> {
    crate::noise_xml::validate(&value.pressure, "<air_speed>")
}
impl AirSpeed {
    /// Emit current pressure Noise without calculating speed or atmosphere.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        Ok(format!(
            "<air_speed><pressure>{}</pressure></air_speed>",
            crate::noise_xml::emit(&self.pressure)
        ))
    }
}
