// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Lidar scan metadata output; consumers decide whether to generate rays.
use crate::{Diagnostic, ErrorCode, Lidar, NoiseKind, Source};

fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<lidar>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
pub(crate) fn validate_config(value: &Lidar) -> Result<(), Diagnostic> {
    let noise = &value.noise;
    if [
        noise.bias_mean,
        noise.bias_stddev,
        noise.dynamic_bias_stddev,
        noise.dynamic_bias_correlation_time,
        noise.precision,
    ]
    .iter()
    .any(|n| n.to_bits() != 0)
    {
        return Err(error(
            ErrorCode::Unsupported,
            "lidar XML cannot preserve advanced Noise fields",
        ));
    }
    if [
        value.horizontal.resolution,
        value.horizontal.min_angle,
        value.horizontal.max_angle,
        value.vertical.resolution,
        value.vertical.min_angle,
        value.vertical.max_angle,
        value.range_min,
        value.range_max,
        value.range_resolution,
        noise.mean,
        noise.stddev,
    ]
    .iter()
    .any(|n| !n.is_finite() || n.is_subnormal())
        || noise.stddev < 0.
    {
        return Err(error(
            ErrorCode::InvalidValue,
            "lidar values cannot roundtrip through the supported XML reader",
        ));
    }
    Ok(())
}
impl Lidar {
    /// Emit current scan/range metadata without adding runtime ordering rules.
    /// Advanced Noise payloads that this grammar cannot retain return Unsupported.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate_config(self)?;
        let mut output = String::from("<lidar><scan>");
        for (name, scan) in [
            ("horizontal", &self.horizontal),
            ("vertical", &self.vertical),
        ] {
            output.push_str(&format!(
                "<{name}><samples>{}</samples><resolution>{}</resolution><min_angle>{}</min_angle><max_angle>{}</max_angle></{name}>",
                scan.samples, scan.resolution, scan.min_angle, scan.max_angle
            ));
        }
        let kind = match self.noise.kind {
            NoiseKind::None => "none",
            NoiseKind::Gaussian => "gaussian",
            NoiseKind::GaussianQuantized => "gaussian_quantized",
        };
        output.push_str(&format!(
            "</scan><range><min>{}</min><max>{}</max><resolution>{}</resolution></range><noise><type>{kind}</type><mean>{}</mean><stddev>{}</stddev></noise><visibility_mask>{}</visibility_mask></lidar>",
            self.range_min, self.range_max, self.range_resolution,
            self.noise.mean, self.noise.stddev, self.visibility_mask
        ));
        Ok(output)
    }
}
