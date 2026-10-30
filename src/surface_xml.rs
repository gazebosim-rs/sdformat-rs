// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Supported surface metadata; consumers implement or reject its physical use.
use crate::{Diagnostic, ErrorCode, Source, Surface};

fn error() -> Diagnostic {
    Diagnostic {
        code: ErrorCode::InvalidValue,
        source: Source {
            uri: "<surface>".into(),
            line: 0,
            column: 0,
        },
        message: "surface value cannot roundtrip through the supported XML reader".into(),
    }
}
fn scalar(name: &str, value: f64, nonnegative: bool) -> Result<String, Diagnostic> {
    if !value.is_finite() || value.is_subnormal() || (nonnegative && value < 0.) {
        return Err(error());
    }
    Ok(format!("<{name}>{value}</{name}>"))
}
fn vector(value: [f64; 3]) -> Result<String, Diagnostic> {
    if value.iter().any(|value| !value.is_finite()) {
        return Err(error());
    }
    Ok(format!(
        "<fdir1>{} {} {}</fdir1>",
        value[0], value[1], value[2]
    ))
}
impl Surface {
    /// Emit the supported masks and friction groups; origin metadata is unchanged.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        let category = self
            .contact
            .category_bitmask
            .map_or_else(String::new, |mask| {
                format!("<category_bitmask>{mask}</category_bitmask>")
            });
        let ode = &self.friction.ode;
        let ode = format!(
            "<ode>{}{}{}{}{}</ode>",
            scalar("mu", ode.mu, true)?,
            scalar("mu2", ode.mu2, true)?,
            scalar("slip1", ode.slip1, false)?,
            scalar("slip2", ode.slip2, false)?,
            vector(ode.fdir1)?
        );
        let bullet = self
            .friction
            .bullet
            .as_ref()
            .map(|v| {
                Ok::<_, Diagnostic>(format!(
                    "<bullet>{}{}{}{}</bullet>",
                    scalar("friction", v.friction, true)?,
                    scalar("friction2", v.friction2, true)?,
                    vector(v.fdir1)?,
                    scalar("rolling_friction", v.rolling_friction, false)?
                ))
            })
            .transpose()?
            .unwrap_or_default();
        let torsional = self.friction.torsional.as_ref().map(|v| {
            Ok::<_, Diagnostic>(format!("<torsional>{}<use_patch_radius>{}</use_patch_radius>{}{}<ode>{}</ode></torsional>",
                scalar("coefficient",v.coefficient,true)?, v.use_patch_radius,
                scalar("patch_radius",v.patch_radius,true)?,scalar("surface_radius",v.surface_radius,true)?,
                scalar("slip",v.ode_slip,false)?))
        }).transpose()?.unwrap_or_default();
        Ok(format!(
            "<surface><contact><collide_bitmask>{}</collide_bitmask>{category}</contact><friction>{ode}{bullet}{torsional}</friction></surface>",
            self.contact.collide_bitmask
        ))
    }
}
