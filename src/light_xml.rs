// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

//! Bounded standalone Light XML output. Document-level Light grammar is separate.
use crate::{Diagnostic, ErrorCode, Light, LightType, Source};
use quick_xml::escape::escape;

fn safe_attribute(value: &str) -> bool {
    // Attribute whitespace is normalized by XML readers; forbidden code points
    // cannot be escaped into a faithful owned string roundtrip.
    value
        .chars()
        .all(|c| c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
}

impl Light {
    /// Serialize owned Light values as one SDF 1.12 `<light>` element.
    /// The quaternion pose form preserves the owned rotation through a roundtrip.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let source = Source {
            uri: "<light>".into(),
            line: 0,
            column: 0,
        };
        self.pose.transform.validate(&source)?;
        let scalars = [
            self.intensity,
            self.attenuation_range,
            self.linear_attenuation_factor,
            self.constant_attenuation_factor,
            self.quadratic_attenuation_factor,
            self.spot_inner_angle,
            self.spot_outer_angle,
            self.spot_falloff,
        ];
        let reserved_name = self.name == "world"
            || (self.name.len() >= 4 && self.name.starts_with("__") && self.name.ends_with("__"));
        // Shortest decimal subnormals trigger native scalar/pose ERANGE, even
        // though the caller's stored value is finite.
        let pose_subnormal = self
            .pose
            .transform
            .translation
            .iter()
            .chain(&self.pose.transform.rotation)
            .any(|v| v.is_subnormal());
        if self.name.is_empty()
            || self.name.trim() != self.name
            || self.name.contains("::")
            || reserved_name
            || !safe_attribute(&self.name)
            || !safe_attribute(&self.pose.relative_to)
            || pose_subnormal
            || scalars.iter().any(|v| !v.is_finite() || v.is_subnormal())
            || self.direction.iter().any(|v| !v.is_finite())
            || self
                .diffuse
                .iter()
                .chain(&self.specular)
                .any(|v| !v.is_finite() || v.is_subnormal() || !(0. ..=1.).contains(v))
            || self.attenuation_range < 0.
            || !(0. ..=1.).contains(&self.linear_attenuation_factor)
            || !(0. ..=1.).contains(&self.constant_attenuation_factor)
            || self.quadratic_attenuation_factor < 0.
            || self.spot_inner_angle < 0.
            || self.spot_outer_angle < 0.
            || self.spot_falloff < 0.
        {
            return Err(Diagnostic {
                code: ErrorCode::InvalidValue,
                source,
                message: "light values cannot roundtrip through bounded XML".into(),
            });
        }
        let kind = match self.light_type {
            LightType::Point => "point",
            LightType::Directional => "directional",
            LightType::Spot => "spot",
        };
        let p = self.pose.transform.translation;
        let q = self.pose.transform.rotation;
        let pose = if self.pose.relative_to.is_empty() {
            format!(
                "<pose rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose>",
                p[0], p[1], p[2], q[0], q[1], q[2], q[3]
            )
        } else {
            format!(
                "<pose relative_to=\"{}\" rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose>",
                escape(&self.pose.relative_to),
                p[0],
                p[1],
                p[2],
                q[0],
                q[1],
                q[2],
                q[3]
            )
        };
        Ok(format!(
            "<light name=\"{}\" type=\"{}\">{}<light_on>{}</light_on><visualize>{}</visualize>\
             <cast_shadows>{}</cast_shadows><intensity>{}</intensity>\
             <diffuse>{} {} {} {}</diffuse><specular>{} {} {} {}</specular>\
             <attenuation><range>{}</range><linear>{}</linear><constant>{}</constant><quadratic>{}</quadratic></attenuation>\
             <direction>{} {} {}</direction>\
             <spot><inner_angle>{}</inner_angle><outer_angle>{}</outer_angle><falloff>{}</falloff></spot></light>",
            escape(&self.name),
            kind,
            pose,
            self.light_on,
            self.visualize,
            self.cast_shadows,
            self.intensity,
            self.diffuse[0],
            self.diffuse[1],
            self.diffuse[2],
            self.diffuse[3],
            self.specular[0],
            self.specular[1],
            self.specular[2],
            self.specular[3],
            self.attenuation_range,
            self.linear_attenuation_factor,
            self.constant_attenuation_factor,
            self.quadratic_attenuation_factor,
            self.direction[0],
            self.direction[1],
            self.direction[2],
            self.spot_inner_angle,
            self.spot_outer_angle,
            self.spot_falloff,
        ))
    }
}
