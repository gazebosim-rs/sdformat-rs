// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Noise XML shared by the supported typed sensor families.
use crate::{Diagnostic, ErrorCode, Noise, NoiseKind, Source};
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
pub(crate) fn validate(noise: &Noise, source: &str) -> Result<(), Diagnostic> {
    if noise_fields(noise)
        .iter()
        .any(|(_, value)| !value.is_finite() || value.is_subnormal())
        || noise.stddev < 0.
        || noise.bias_stddev < 0.
        || noise.dynamic_bias_stddev < 0.
    {
        return Err(Diagnostic {
            code: ErrorCode::InvalidValue,
            source: Source {
                uri: source.into(),
                line: 0,
                column: 0,
            },
            message: "noise values cannot roundtrip through the supported XML reader".into(),
        });
    }
    Ok(())
}
pub(crate) fn emit(noise: &Noise) -> String {
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
