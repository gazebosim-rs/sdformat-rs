// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Mesh metadata output; URI resolution and asset processing remain separate.
use crate::{Diagnostic, ErrorCode, Mesh, MeshOptimization, Source};
fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<mesh>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn validate_text(value: &str) -> Result<(), Diagnostic> {
    crate::param_xml::validate_text(value).map_err(|code| {
        let message = if code == ErrorCode::Unsupported {
            "native Param would truncate split CDATA"
        } else {
            "mesh text cannot roundtrip through XML"
        };
        error(code, message)
    })
}

impl Mesh {
    /// Emit current URI/scale/submesh/optimization metadata without resolving it.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_text(&self.uri)?;
        if self.scale.iter().any(|v| !v.is_finite()) {
            return Err(error(ErrorCode::InvalidValue, "mesh scale must be finite"));
        }
        if (self.submesh.is_empty() && self.center_submesh) || self.submesh == "__default__" {
            return Err(error(
                ErrorCode::Unsupported,
                "submesh state cannot roundtrip through XML",
            ));
        }
        let submesh = if self.submesh.is_empty() {
            String::new()
        } else {
            validate_text(&self.submesh)?;
            format!(
                "<submesh><name>{}</name><center>{}</center></submesh>",
                crate::param_xml::text(&self.submesh),
                self.center_submesh
            )
        };
        let optimization = match self.optimization {
            MeshOptimization::None => "",
            MeshOptimization::ConvexHull => "convex_hull",
            MeshOptimization::ConvexDecomposition => "convex_decomposition",
        };
        let decomposition=self.convex_decomposition.map_or_else(String::new,|v|format!("<convex_decomposition><max_convex_hulls>{}</max_convex_hulls><voxel_resolution>{}</voxel_resolution></convex_decomposition>",v.max_convex_hulls,v.voxel_resolution));
        Ok(format!(
            "<mesh optimization=\"{optimization}\"><uri>{}</uri><scale>{} {} {}</scale>{submesh}{decomposition}</mesh>",
            crate::param_xml::text(&self.uri),
            self.scale[0],
            self.scale[1],
            self.scale[2]
        ))
    }
}
