// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Navigation noise metadata; position and velocity estimates belong to consumers.
use crate::{Diagnostic, NavSat};

pub(crate) fn validate_config(value: &NavSat) -> Result<(), Diagnostic> {
    for noise in [
        &value.horizontal_position,
        &value.vertical_position,
        &value.horizontal_velocity,
        &value.vertical_velocity,
    ] {
        crate::noise_xml::validate(noise, "<navsat>")?;
    }
    Ok(())
}
impl NavSat {
    /// Emit four Noise records without calculating navigation measurements.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        let mut output = String::from("<navsat>");
        for (group, horizontal, vertical) in [
            (
                "position_sensing",
                &self.horizontal_position,
                &self.vertical_position,
            ),
            (
                "velocity_sensing",
                &self.horizontal_velocity,
                &self.vertical_velocity,
            ),
        ] {
            output.push_str(&format!(
                "<{group}><horizontal>{}</horizontal><vertical>{}</vertical></{group}>",
                crate::noise_xml::emit(horizontal),
                crate::noise_xml::emit(vertical)
            ));
        }
        output.push_str("</navsat>");
        Ok(output)
    }
}
