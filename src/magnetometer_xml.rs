// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Magnetometer noise metadata; field measurement belongs to consumers.
use crate::{Diagnostic, Magnetometer};

pub(crate) fn validate_config(value: &Magnetometer) -> Result<(), Diagnostic> {
    for noise in &value.noise {
        crate::noise_xml::validate(noise, "<magnetometer>")?;
    }
    Ok(())
}
impl Magnetometer {
    /// Emit all three noise records without computing a magnetic field.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        let mut output = String::from("<magnetometer>");
        for (axis, noise) in ["x", "y", "z"].iter().zip(&self.noise) {
            output.push_str(&format!(
                "<{axis}>{}</{axis}>",
                crate::noise_xml::emit(noise)
            ));
        }
        output.push_str("</magnetometer>");
        Ok(output)
    }
}
