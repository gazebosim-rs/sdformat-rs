// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Current-schema axis metadata output, independent of frame or mimic graphs.
use crate::types::normalized_axis_direction;
use crate::{Axis, Diagnostic, ErrorCode, Source};
use quick_xml::escape::escape;

fn safe_attribute(value: &str) -> bool {
    value.trim_matches([' ', '\t', '\n']) == value
        && value
            .chars()
            .all(|c| c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
}
fn decimal(value: f64) -> String {
    if value.is_nan() && value.is_sign_negative() {
        "-nan".into()
    } else {
        value.to_string()
    }
}
impl Axis {
    /// Emit one current-schema `<axis>` element, normalizing its direction.
    /// Negative effort/velocity encode the native unbounded sentinel. Graph
    /// admission remains the responsibility of the containing model loader.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let invalid = || Diagnostic {
            code: ErrorCode::InvalidValue,
            source: Source {
                uri: "<axis>".into(),
                line: 0,
                column: 0,
            },
            message: "axis values cannot roundtrip through bounded XML".into(),
        };
        let xyz = normalized_axis_direction(self.xyz).map_err(|_| invalid())?;
        let finite = [
            self.dynamics.damping,
            self.dynamics.friction,
            self.dynamics.spring_reference,
            self.dynamics.spring_stiffness,
            self.stiffness,
            self.dissipation,
        ];
        if !safe_attribute(&self.expressed_in)
            || finite.iter().any(|v| !v.is_finite() || v.is_subnormal())
            || xyz.iter().any(|v| v.is_subnormal())
            || [self.lower, self.upper, self.effort, self.velocity]
                .iter()
                .any(|v| v.is_subnormal())
        {
            return Err(invalid());
        }
        let mimic = if let Some(mimic) = &self.mimic {
            if mimic.joint.trim().is_empty()
                || !safe_attribute(&mimic.joint)
                || !safe_attribute(&mimic.axis)
                || [mimic.multiplier, mimic.offset, mimic.reference]
                    .iter()
                    .any(|v| !v.is_finite() || v.is_subnormal())
            {
                return Err(invalid());
            }
            format!(
                "<mimic joint=\"{}\" axis=\"{}\"><multiplier>{}</multiplier><offset>{}</offset><reference>{}</reference></mimic>",
                escape(&mimic.joint),
                escape(&mimic.axis),
                mimic.multiplier,
                mimic.offset,
                mimic.reference
            )
        } else {
            String::new()
        };
        let unbounded = |value: f64| if value < 0. { f64::INFINITY } else { value };
        Ok(format!(
            "<axis><xyz expressed_in=\"{}\">{} {} {}</xyz>\
            <dynamics><damping>{}</damping><friction>{}</friction><spring_reference>{}</spring_reference><spring_stiffness>{}</spring_stiffness></dynamics>\
            <limit><lower>{}</lower><upper>{}</upper><effort>{}</effort><velocity>{}</velocity><stiffness>{}</stiffness><dissipation>{}</dissipation></limit>{}</axis>",
            escape(&self.expressed_in),
            xyz[0],
            xyz[1],
            xyz[2],
            self.dynamics.damping,
            self.dynamics.friction,
            self.dynamics.spring_reference,
            self.dynamics.spring_stiffness,
            decimal(self.lower),
            decimal(self.upper),
            decimal(unbounded(self.effort)),
            decimal(unbounded(self.velocity)),
            self.stiffness,
            self.dissipation,
            mimic
        ))
    }
}
