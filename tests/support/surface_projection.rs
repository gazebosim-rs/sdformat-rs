// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::Surface;
pub fn projection(surface: &Surface) -> Vec<String> {
    let mut values = vec![
        "OK".into(),
        surface.contact.collide_bitmask.to_string(),
        surface
            .contact
            .category_bitmask
            .map_or_else(|| "-".into(), |v| v.to_string()),
    ];
    let ode = &surface.friction.ode;
    values.extend(
        [ode.mu, ode.mu2, ode.slip1, ode.slip2]
            .into_iter()
            .chain(ode.fdir1)
            .map(|v| v.to_bits().to_string()),
    );
    values.push(u8::from(surface.friction.bullet.is_some()).to_string());
    if let Some(v) = &surface.friction.bullet {
        values.extend(
            [v.friction, v.friction2, v.rolling_friction]
                .into_iter()
                .chain(v.fdir1)
                .map(|v| v.to_bits().to_string()),
        );
    }
    values.push(u8::from(surface.friction.torsional.is_some()).to_string());
    if let Some(v) = &surface.friction.torsional {
        values.extend(
            [v.coefficient, v.patch_radius, v.surface_radius, v.ode_slip]
                .map(|v| v.to_bits().to_string()),
        );
        values.push(u8::from(v.use_patch_radius).to_string());
    }
    values
}
