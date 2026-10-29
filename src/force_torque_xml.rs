// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Typed force/torque configuration output; no sampling or graph resolution.
use crate::{Diagnostic, ForceTorque, ForceTorqueFrame, ForceTorqueMeasureDirection};
pub(crate) fn validate_config(value: &ForceTorque) -> Result<(), Diagnostic> {
    for noise in value.force.iter().chain(&value.torque) {
        crate::noise_xml::validate(noise, "<force_torque>")?;
    }
    Ok(())
}
impl ForceTorque {
    /// Emit current frame/direction and six noise records without changing provenance.
    /// Existing finite-noise and standard-deviation admission rules remain in force.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        let frame = match self.frame {
            ForceTorqueFrame::Parent => "parent",
            ForceTorqueFrame::Child => "child",
            ForceTorqueFrame::Sensor => "sensor",
        };
        let direction = match self.measure_direction {
            ForceTorqueMeasureDirection::ParentToChild => "parent_to_child",
            ForceTorqueMeasureDirection::ChildToParent => "child_to_parent",
        };
        let mut output = format!(
            "<force_torque><frame>{frame}</frame><measure_direction>{direction}</measure_direction>"
        );
        for (group, noises) in [("force", &self.force), ("torque", &self.torque)] {
            output.push_str(&format!("<{group}>"));
            for (axis, noise) in ["x", "y", "z"].iter().zip(noises) {
                output.push_str(&format!(
                    "<{axis}>{}</{axis}>",
                    crate::noise_xml::emit(noise)
                ));
            }
            output.push_str(&format!("</{group}>"));
        }
        output.push_str("</force_torque>");
        Ok(output)
    }
}
