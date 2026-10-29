// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! IMU metadata output; vectors are retained without normalization.
use crate::{Diagnostic, ErrorCode, Imu, Source};
use quick_xml::escape::escape;
fn attribute(value: &str) -> bool {
    value.trim() == value
        && value
            .chars()
            .all(|c| c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
}
pub(crate) fn validate_config(imu: &Imu) -> Result<(), Diagnostic> {
    if imu.localization.is_empty()
        || !attribute(&imu.localization)
        || imu.localization.contains("  ")
        || !attribute(&imu.custom_rpy_parent_frame)
        || !attribute(&imu.gravity_dir_x_parent_frame)
        || imu
            .custom_rpy
            .iter()
            .chain(&imu.gravity_dir_x)
            .any(|n| !n.is_finite() || n.is_subnormal())
    {
        return Err(Diagnostic {
            code: ErrorCode::InvalidValue,
            source: Source {
                uri: "<imu>".into(),
                line: 0,
                column: 0,
            },
            message: "IMU reference metadata cannot roundtrip through XML".into(),
        });
    }
    for noise in imu.angular_velocity.iter().chain(&imu.linear_acceleration) {
        crate::noise_xml::validate(noise, "<imu>")?;
    }
    Ok(())
}
impl Imu {
    /// Emit orientation metadata and six noise records without resolving frames.
    /// Gravity and custom RPY remain raw vectors, including zero gravity.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        let r = self.custom_rpy;
        let g = self.gravity_dir_x;
        let mut output = format!(
            "<imu><enable_orientation>{}</enable_orientation><orientation_reference_frame><localization>{}</localization><custom_rpy parent_frame=\"{}\">{} {} {}</custom_rpy><grav_dir_x parent_frame=\"{}\">{} {} {}</grav_dir_x></orientation_reference_frame>",
            self.enable_orientation,
            escape(&self.localization),
            escape(&self.custom_rpy_parent_frame),
            r[0],
            r[1],
            r[2],
            escape(&self.gravity_dir_x_parent_frame),
            g[0],
            g[1],
            g[2]
        );
        for (group, noises) in [
            ("angular_velocity", &self.angular_velocity),
            ("linear_acceleration", &self.linear_acceleration),
        ] {
            output.push_str(&format!("<{group}>"));
            for (axis, noise) in ["x", "y", "z"].iter().zip(noises) {
                output.push_str(&format!(
                    "<{axis}>{}</{axis}>",
                    crate::noise_xml::emit(noise)
                ));
            }
            output.push_str(&format!("</{group}>"));
        }
        output.push_str("</imu>");
        Ok(output)
    }
}
