// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Current Material/PBR metadata; no script, texture or renderer is loaded.
use crate::{Diagnostic, ErrorCode, Material, PbrWorkflow, PbrWorkflowKind, ShaderType, Source};
use quick_xml::escape::escape;
fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<material>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn text(value: &str) -> Result<String, Diagnostic> {
    if !value.is_empty() {
        crate::param_xml::validate_text(value)
            .map_err(|code| error(code, "material text cannot roundtrip through native Param"))?;
    }
    Ok(crate::param_xml::text(value))
}
fn optional(name: &str, value: &str) -> Result<String, Diagnostic> {
    if value.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!("<{name}>{}</{name}>", text(value)?))
    }
}
fn attribute(value: &str) -> Result<String, Diagnostic> {
    if value
        .chars()
        .any(|c| c < '\u{20}' || matches!(c, '\u{fffe}' | '\u{ffff}'))
    {
        return Err(error(
            ErrorCode::Unsupported,
            "raw attribute contains characters outside the lossless output domain",
        ));
    }
    Ok(escape(value).into_owned())
}
fn float(value: f32) -> Result<String, Diagnostic> {
    let output = if value.is_nan() && value.is_sign_negative() {
        "-nan".into()
    } else {
        value.to_string()
    };
    let loaded = crate::number::float(&output)
        .map_err(|code| error(code, "material float conversion failed"))?;
    if loaded.to_bits() != value.to_bits() {
        return Err(error(
            ErrorCode::Unsupported,
            "float payload cannot roundtrip through XML",
        ));
    }
    Ok(output)
}
fn double(value: f64, metadata: bool) -> Result<String, Diagnostic> {
    let output = crate::param_xml::decimal(value);
    let loaded = if metadata && !value.is_finite() {
        if value.is_nan() {
            if value.is_sign_negative() {
                -f64::NAN
            } else {
                f64::NAN
            }
        } else {
            value
        }
    } else {
        crate::number::scalar_float(&output)
            .map_err(|code| error(code, "material scalar conversion failed"))?
    };
    if loaded.to_bits() != value.to_bits() {
        return Err(error(
            ErrorCode::Unsupported,
            "scalar payload cannot roundtrip through XML",
        ));
    }
    Ok(output)
}
fn color(name: &str, value: [f32; 4]) -> Result<String, Diagnostic> {
    if value
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        return Err(error(
            ErrorCode::InvalidValue,
            "material colors must be finite and within zero to one",
        ));
    }
    let components = value
        .into_iter()
        .map(float)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("<{name}>{}</{name}>", components.join(" ")))
}
fn workflow(value: &PbrWorkflow, kind: PbrWorkflowKind) -> Result<String, Diagnostic> {
    if value.kind != kind {
        return Err(error(
            ErrorCode::Unsupported,
            "PBR workflow kind does not match its slot",
        ));
    }
    let metal = kind == PbrWorkflowKind::Metal;
    if (metal
        && (!value.specular_map.is_empty()
            || !value.glossiness_map.is_empty()
            || value.glossiness.to_bits() != 0.0_f64.to_bits()))
        || (!metal
            && (!value.roughness_map.is_empty()
                || !value.metalness_map.is_empty()
                || value.roughness.to_bits() != 0.5_f64.to_bits()
                || value.metalness.to_bits() != 0.5_f64.to_bits()))
    {
        return Err(error(
            ErrorCode::Unsupported,
            "inactive workflow fields cannot be retained by the typed XML reader",
        ));
    }
    let tag = if metal { "metal" } else { "specular" };
    let mut xml = format!("<{tag}>");
    for (name, path) in [
        ("albedo_map", &value.albedo_map),
        ("environment_map", &value.environment_map),
        ("ambient_occlusion_map", &value.ambient_occlusion_map),
        ("emissive_map", &value.emissive_map),
    ] {
        xml.push_str(&optional(name, path)?);
    }
    if !value.normal_map.is_empty() || value.normal_map_type.is_some() {
        let attr = value
            .normal_map_type
            .as_deref()
            .map(|v| attribute(v).map(|s| format!(" type=\"{s}\"")))
            .transpose()?
            .unwrap_or_default();
        xml.push_str(&format!(
            "<normal_map{attr}>{}</normal_map>",
            text(&value.normal_map)?
        ));
    }
    if !value.light_map.is_empty() || value.light_map_uv_set != 0 {
        xml.push_str(&format!(
            "<light_map uv_set=\"{}\">{}</light_map>",
            value.light_map_uv_set,
            text(&value.light_map)?
        ));
    }
    if metal {
        xml.push_str(&optional("roughness_map", &value.roughness_map)?);
        xml.push_str(&optional("metalness_map", &value.metalness_map)?);
        xml.push_str(&format!(
            "<roughness>{}</roughness><metalness>{}</metalness>",
            double(value.roughness, true)?,
            double(value.metalness, true)?
        ));
    } else {
        xml.push_str(&optional("specular_map", &value.specular_map)?);
        xml.push_str(&optional("glossiness_map", &value.glossiness_map)?);
        xml.push_str(&format!(
            "<glossiness>{}</glossiness>",
            double(value.glossiness, true)?
        ));
    }
    xml.push_str(&format!("</{tag}>"));
    Ok(xml)
}
impl Material {
    /// Emit losslessly representable current metadata; input origin is unchanged.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let mut xml = "<material>".to_owned();
        for (name, value) in [
            ("ambient", self.ambient),
            ("diffuse", self.diffuse),
            ("specular", self.specular),
            ("emissive", self.emissive),
        ] {
            xml.push_str(&color(name, value)?);
        }
        xml.push_str(&format!("<shininess>{}</shininess><render_order>{}</render_order><lighting>{}</lighting><double_sided>{}</double_sided>",double(self.shininess,false)?,float(self.render_order)?,self.lighting,self.double_sided));
        if let Some(script) = &self.script {
            if script.uris.is_empty()
                || script.uris[0].is_empty()
                || script.uris[0] == "__default__"
                || script.name.is_empty()
                || script.name == "__default__"
            {
                return Err(error(
                    ErrorCode::Unsupported,
                    "material script requires a representable URI and name",
                ));
            }
            xml.push_str("<script>");
            for uri in &script.uris {
                if uri.is_empty() {
                    return Err(error(
                        ErrorCode::Unsupported,
                        "empty script URI would gain a schema default",
                    ));
                }
                xml.push_str(&format!("<uri>{}</uri>", text(uri)?));
            }
            xml.push_str(&format!("<name>{}</name></script>", text(&script.name)?));
        }
        let shader = match self.shader {
            ShaderType::Pixel => "pixel",
            ShaderType::Vertex => "vertex",
            ShaderType::NormalMapObjectSpace => "normal_map_object_space",
            ShaderType::NormalMapTangentSpace => "normal_map_tangent_space",
        };
        if self.normal_map == "__default__"
            || (matches!(
                self.shader,
                ShaderType::NormalMapObjectSpace | ShaderType::NormalMapTangentSpace
            ) && self.normal_map.is_empty())
        {
            return Err(error(
                ErrorCode::Unsupported,
                "normal-map shader metadata would change on reload",
            ));
        }
        xml.push_str(&format!(
            "<shader type=\"{shader}\">{}</shader>",
            optional("normal_map", &self.normal_map)?
        ));
        if let Some(pbr) = &self.pbr {
            xml.push_str("<pbr>");
            if let Some(metal) = &pbr.metal {
                xml.push_str(&workflow(metal, PbrWorkflowKind::Metal)?);
            }
            if let Some(specular) = &pbr.specular {
                xml.push_str(&workflow(specular, PbrWorkflowKind::Specular)?);
            }
            xml.push_str("</pbr>");
        }
        xml.push_str("</material>");
        Ok(xml)
    }
}
