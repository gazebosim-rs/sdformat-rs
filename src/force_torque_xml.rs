// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Typed force/torque configuration output; no sampling or graph resolution.
use crate::{
    Diagnostic, ErrorCode, ForceTorque, ForceTorqueFrame, ForceTorqueMeasureDirection, Noise,
    NoiseKind, Source,
};
fn invalid(message: &str) -> Diagnostic {
    Diagnostic {
        code: ErrorCode::InvalidValue,
        source: Source {
            uri: "<force_torque>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn noise_fields(noise: &Noise) -> [(&'static str, f64); 7] {
    [
        ("mean", noise.mean),
        ("stddev", noise.stddev),
        ("bias_mean", noise.bias_mean),
        ("bias_stddev", noise.bias_stddev),
        ("dynamic_bias_stddev", noise.dynamic_bias_stddev),
        (
            "dynamic_bias_correlation_time",
            noise.dynamic_bias_correlation_time,
        ),
        ("precision", noise.precision),
    ]
}
pub(crate) fn validate_config(value: &ForceTorque) -> Result<(), Diagnostic> {
    for noise in value.force.iter().chain(&value.torque) {
        if noise_fields(noise)
            .iter()
            .any(|(_, value)| !value.is_finite() || value.is_subnormal())
            || noise.stddev < 0.
            || noise.bias_stddev < 0.
            || noise.dynamic_bias_stddev < 0.
        {
            return Err(invalid(
                "noise values cannot roundtrip through the supported XML reader",
            ));
        }
    }
    Ok(())
}
fn noise_xml(noise: &Noise) -> String {
    let fields = noise_fields(noise);
    let kind = match noise.kind {
        NoiseKind::None => "none",
        NoiseKind::Gaussian => "gaussian",
        NoiseKind::GaussianQuantized => "gaussian_quantized",
    };
    let fields = fields
        .iter()
        .map(|(name, value)| format!("<{name}>{value}</{name}>"))
        .collect::<String>();
    format!("<noise type=\"{kind}\">{fields}</noise>")
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
                output.push_str(&format!("<{axis}>{}</{axis}>", noise_xml(noise)));
            }
            output.push_str(&format!("</{group}>"));
        }
        output.push_str("</force_torque>");
        Ok(output)
    }
}
