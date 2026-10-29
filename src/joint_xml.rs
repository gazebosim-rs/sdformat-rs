// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Bounded joint metadata XML; containing models own graph validation.
use crate::{Diagnostic, ErrorCode, Joint, JointKind, Source};
use quick_xml::escape::escape;

fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<joint>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn attribute(value: &str) -> bool {
    value.trim_matches([' ', '\t', '\n']) == value
        && value
            .chars()
            .all(|c| c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
}
fn reference(value: &str) -> bool {
    !value.is_empty() && value.trim() == value && attribute(value) && !value.contains("  ")
}
pub(crate) fn validate_boundary(joint: &Joint) -> Result<(), Diagnostic> {
    if !joint.sensors.is_empty()
        || joint.physics_xml.is_some()
        || (joint.axis.is_none() && joint.axis2.is_some())
        || (joint.kind != JointKind::Screw && joint.screw_thread_pitch != 1.)
    {
        return Err(error(
            ErrorCode::Unsupported,
            "joint XML cannot preserve this payload, axis order or non-screw pitch",
        ));
    }
    let reserved = joint.name == "world"
        || (joint.name.len() >= 4 && joint.name.starts_with("__") && joint.name.ends_with("__"));
    if !reference(&joint.name)
        || joint.name.contains("::")
        || reserved
        || !reference(&joint.parent)
        || !reference(&joint.child)
        || joint.parent == "__root__"
        || joint.child == "__root__"
        || joint.child == "world"
        || joint.parent == joint.child
        || !attribute(&joint.pose.relative_to)
        || !joint.screw_thread_pitch.is_finite()
        || joint.screw_thread_pitch.is_subnormal()
    {
        return Err(error(
            ErrorCode::InvalidValue,
            "invalid standalone joint metadata",
        ));
    }
    for (name, axis) in [("axis", &joint.axis), ("axis2", &joint.axis2)] {
        if let Some(mimic) = axis.as_ref().and_then(|axis| axis.mimic.as_ref())
            && (!matches!(mimic.axis.as_str(), "axis" | "axis2")
                || (mimic.joint == joint.name && mimic.axis == name))
        {
            return Err(error(
                ErrorCode::JointAxisMimicInvalid,
                "invalid local joint mimic declaration",
            ));
        }
    }
    Ok(())
}
impl Joint {
    /// Emit bounded current-schema metadata. Unsupported retained payloads are
    /// errors; serialization does not attach or validate a model graph.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_boundary(self)?;
        let source = Source {
            uri: "<joint>".into(),
            line: 0,
            column: 0,
        };
        self.pose.transform.validate(&source)?;
        if self
            .pose
            .transform
            .translation
            .iter()
            .chain(&self.pose.transform.rotation)
            .any(|value| value.is_subnormal())
        {
            return Err(error(
                ErrorCode::InvalidValue,
                "joint pose cannot roundtrip through XML",
            ));
        }
        let kind = match self.kind {
            JointKind::Fixed => "fixed",
            JointKind::Ball => "ball",
            JointKind::Gearbox => "gearbox",
            JointKind::Screw => "screw",
            JointKind::Revolute => "revolute",
            JointKind::Prismatic => "prismatic",
            JointKind::Continuous => "continuous",
            JointKind::Universal => "universal",
            JointKind::Revolute2 => "revolute2",
        };
        let mut axes = String::new();
        if let Some(axis) = &self.axis {
            axes.push_str(&axis.to_xml_for_slot(false)?);
        }
        if let Some(axis) = &self.axis2 {
            axes.push_str(&axis.to_xml_for_slot(true)?);
        }
        let screw = if self.kind == JointKind::Screw {
            format!(
                "<screw_thread_pitch>{}</screw_thread_pitch>",
                self.screw_thread_pitch
            )
        } else {
            String::new()
        };
        let pose = self.pose.transform;
        let p = pose.translation;
        let q = pose.rotation;
        Ok(format!(
            "<joint name=\"{}\" type=\"{kind}\"><parent>{}</parent><child>{}</child>\
            <pose relative_to=\"{}\" rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose>{axes}{screw}</joint>",
            escape(&self.name),
            escape(&self.parent),
            escape(&self.child),
            escape(&self.pose.relative_to),
            p[0],
            p[1],
            p[2],
            q[0],
            q[1],
            q[2],
            q[3]
        ))
    }
}
