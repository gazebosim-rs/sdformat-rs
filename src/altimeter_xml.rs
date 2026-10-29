// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Altimeter noise configuration; measurement belongs to the consumer.
use crate::{Altimeter, Diagnostic};

pub(crate) fn validate_config(value: &Altimeter) -> Result<(), Diagnostic> {
    crate::noise_xml::validate(&value.vertical_position, "<altimeter>")?;
    crate::noise_xml::validate(&value.vertical_velocity, "<altimeter>")
}

impl Altimeter {
    /// Emit both noise records without sampling height or velocity.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        Ok(format!(
            "<altimeter><vertical_position>{}</vertical_position><vertical_velocity>{}</vertical_velocity></altimeter>",
            crate::noise_xml::emit(&self.vertical_position),
            crate::noise_xml::emit(&self.vertical_velocity)
        ))
    }
}
