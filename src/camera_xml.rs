// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! Bounded base-camera XML, separate from rendering and advanced optics.
use crate::{
    Camera, CameraDistortion, CameraLens, CameraPixelFormat, Diagnostic, ErrorCode, NoiseKind,
    Source,
};
use quick_xml::escape::escape;
fn error(code: ErrorCode, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        source: Source {
            uri: "<camera>".into(),
            line: 0,
            column: 0,
        },
        message: message.into(),
    }
}
fn text(value: &str) -> bool {
    value.trim() == value
        && !value.contains("  ")
        && value
            .chars()
            .all(|c| c >= '\u{20}' && c != '\u{fffe}' && c != '\u{ffff}')
}
pub(crate) fn validate(camera: &Camera) -> Result<(), Diagnostic> {
    let default = Camera::default();
    let lens = CameraLens {
        has_intrinsics: camera.lens.has_intrinsics,
        has_projection: camera.lens.has_projection,
        cutoff_angle: camera.lens.cutoff_angle,
        ..CameraLens::default()
    };
    let noise = camera.image_noise;
    if camera.has_depth_near_clip
        || camera.has_depth_far_clip
        || camera.depth_near_clip != default.depth_near_clip
        || camera.depth_far_clip != default.depth_far_clip
        || camera.has_segmentation_type
        || camera.segmentation_type != default.segmentation_type
        || camera.has_bounding_box_type
        || camera.bounding_box_type != default.bounding_box_type
        || camera.distortion != CameraDistortion::default()
        || camera.lens != lens
        || ![
            CameraLens::default().cutoff_angle,
            crate::camera::SCHEMA_LENS_CUTOFF,
        ]
        .contains(&camera.lens.cutoff_angle)
        // Omitted zero-valued metadata must be canonical +0, not merely == 0.
        || [
            camera.distortion.k1,
            camera.distortion.k2,
            camera.distortion.k3,
            camera.distortion.p1,
            camera.distortion.p2,
            camera.lens.c3,
            noise.bias_mean,
            noise.bias_stddev,
            noise.dynamic_bias_stddev,
            noise.dynamic_bias_correlation_time,
            noise.precision,
        ]
        .iter()
        .any(|n| n.to_bits() != 0)
        || (!camera.lens.has_intrinsics && camera.lens.intrinsics.skew.to_bits() != 0)
        || (!camera.lens.has_projection && [camera.lens.projection.tx,camera.lens.projection.ty].iter().any(|n|n.to_bits()!=0))
    {
        return Err(error(
            ErrorCode::Unsupported,
            "base camera XML cannot preserve advanced camera payloads",
        ));
    }
    if camera.name.is_empty()
        || [
            &camera.name,
            &camera.pose.relative_to,
            &camera.save_frames_path,
            &camera.camera_info_topic,
            &camera.trigger_topic,
        ]
        .iter()
        .any(|s| !text(s))
        || [
            camera.horizontal_fov,
            camera.near_clip,
            camera.far_clip,
            noise.mean,
            noise.stddev,
        ]
        .iter()
        .any(|n| !n.is_finite() || n.is_subnormal())
        || !(0.001..=6.283186).contains(&camera.horizontal_fov)
        || camera.near_clip < 0.
        || camera.far_clip < 0.1
        || noise.stddev < 0.
        || [
            camera.image_width,
            camera.image_height,
            camera.anti_aliasing,
        ]
        .iter()
        .any(|n| *n > i32::MAX as u32)
        || (camera.save_frames && camera.save_frames_path.is_empty())
        || (!camera.save_frames && !camera.save_frames_path.is_empty())
    {
        return Err(error(
            ErrorCode::InvalidValue,
            "camera values cannot roundtrip through the supported XML boundary",
        ));
    }
    camera.pose.transform.validate(&Source {
        uri: "<camera>".into(),
        line: 0,
        column: 0,
    })?;
    if camera
        .pose
        .transform
        .translation
        .iter()
        .chain(&camera.pose.transform.rotation)
        .any(|n| n.is_subnormal())
    {
        return Err(error(
            ErrorCode::InvalidValue,
            "camera pose cannot roundtrip through XML",
        ));
    }
    Ok(())
}
impl Camera {
    /// Emit base optics/image/clip/save/trigger values. Advanced retained payloads
    /// return Unsupported; serialization never renders or writes image files.
    pub fn to_xml(&self) -> Result<String, Diagnostic> {
        validate(self)?;
        let p = self.pose.transform.translation;
        let q = self.pose.transform.rotation;
        let format = match self.pixel_format {
            CameraPixelFormat::RgbInt8 => "RGB_INT8",
            CameraPixelFormat::LInt8 => "L_INT8",
        };
        let noise = match self.image_noise.kind {
            NoiseKind::None => "none",
            NoiseKind::Gaussian => "gaussian",
            NoiseKind::GaussianQuantized => "gaussian_quantized",
        };
        let mut output = format!(
            "<camera name=\"{}\"><pose relative_to=\"{}\" rotation_format=\"quat_xyzw\">{} {} {} {} {} {} {}</pose><horizontal_fov>{}</horizontal_fov><image><width>{}</width><height>{}</height><format>{format}</format><anti_aliasing>{}</anti_aliasing></image><clip><near>{}</near><far>{}</far></clip><camera_info_topic>{}</camera_info_topic><trigger_topic>{}</trigger_topic><triggered>{}</triggered><visibility_mask>{}</visibility_mask><save enabled=\"{}\">",
            escape(&self.name),
            escape(&self.pose.relative_to),
            p[0],
            p[1],
            p[2],
            q[0],
            q[1],
            q[2],
            q[3],
            self.horizontal_fov,
            self.image_width,
            self.image_height,
            self.anti_aliasing,
            self.near_clip,
            self.far_clip,
            escape(&self.camera_info_topic),
            escape(&self.trigger_topic),
            self.triggered,
            self.visibility_mask,
            self.save_frames
        );
        if self.save_frames {
            output.push_str(&format!("<path>{}</path>", escape(&self.save_frames_path)));
        }
        output.push_str(&format!(
            "</save><noise><type>{noise}</type><mean>{}</mean><stddev>{}</stddev></noise>",
            self.image_noise.mean, self.image_noise.stddev
        ));
        {
            output.push_str(&format!("<lens><type>{}</type><scale_to_hfov>{}</scale_to_hfov><cutoff_angle>{}</cutoff_angle><env_texture_size>{}</env_texture_size>",escape(&self.lens.kind),self.lens.scale_to_hfov,self.lens.cutoff_angle,self.lens.environment_texture_size));
            if self.lens.has_intrinsics {
                let i = &self.lens.intrinsics;
                output.push_str(&format!("<intrinsics><fx>{}</fx><fy>{}</fy><cx>{}</cx><cy>{}</cy><s>{}</s></intrinsics>",i.fx,i.fy,i.cx,i.cy,i.skew));
            }
            if self.lens.has_projection {
                let p = &self.lens.projection;
                output.push_str(&format!("<projection><p_fx>{}</p_fx><p_fy>{}</p_fy><p_cx>{}</p_cx><p_cy>{}</p_cy><tx>{}</tx><ty>{}</ty></projection>",p.fx,p.fy,p.cx,p.cy,p.tx,p.ty));
            }
            output.push_str("</lens>");
        }
        output.push_str("</camera>");
        Ok(output)
    }
}
