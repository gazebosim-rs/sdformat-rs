// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::Pose;

/// Pixel formats observed by the retained camera construction original.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraPixelFormat {
    #[default]
    RgbInt8,
    LInt8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraDistortion {
    pub k1: f64,
    pub k2: f64,
    pub k3: f64,
    pub p1: f64,
    pub p2: f64,
    pub center: [f64; 2],
}

impl Default for CameraDistortion {
    fn default() -> Self {
        Self {
            k1: 0.,
            k2: 0.,
            k3: 0.,
            p1: 0.,
            p2: 0.,
            center: [0.5, 0.5],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraIntrinsics {
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
    pub skew: f64,
}

impl Default for CameraIntrinsics {
    fn default() -> Self {
        Self {
            fx: 277.,
            fy: 277.,
            cx: 160.,
            cy: 120.,
            skew: 0.,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraProjection {
    pub fx: f64,
    pub fy: f64,
    pub cx: f64,
    pub cy: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Default for CameraProjection {
    fn default() -> Self {
        Self {
            fx: 277.,
            fy: 277.,
            cx: 160.,
            cy: 120.,
            tx: 0.,
            ty: 0.,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraLens {
    pub kind: String,
    pub scale_to_hfov: bool,
    pub c1: f64,
    pub c2: f64,
    pub c3: f64,
    pub focal_length: f64,
    pub function: String,
    pub cutoff_angle: f64,
    pub environment_texture_size: u32,
    pub intrinsics: CameraIntrinsics,
    pub projection: CameraProjection,
    pub has_intrinsics: bool,
    pub has_projection: bool,
}

impl Default for CameraLens {
    fn default() -> Self {
        Self {
            kind: "stereographic".into(),
            scale_to_hfov: true,
            c1: 1.,
            c2: 1.,
            c3: 0.,
            focal_length: 1.,
            function: "tan".into(),
            cutoff_angle: std::f64::consts::FRAC_PI_2,
            environment_texture_size: 256,
            intrinsics: CameraIntrinsics::default(),
            projection: CameraProjection::default(),
            has_intrinsics: true,
            has_projection: true,
        }
    }
}

/// Owned camera values. Parsing, serialization and rendering are separate work.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub name: String,
    pub triggered: bool,
    pub trigger_topic: String,
    pub camera_info_topic: String,
    pub horizontal_fov: f64,
    pub image_width: u32,
    pub image_height: u32,
    pub pixel_format: CameraPixelFormat,
    pub anti_aliasing: u32,
    pub depth_near_clip: f64,
    pub has_depth_near_clip: bool,
    pub depth_far_clip: f64,
    pub has_depth_far_clip: bool,
    pub near_clip: f64,
    pub far_clip: f64,
    pub segmentation_type: String,
    pub has_segmentation_type: bool,
    pub bounding_box_type: String,
    pub has_bounding_box_type: bool,
    pub save_frames: bool,
    pub save_frames_path: String,
    pub distortion: CameraDistortion,
    pub pose: Pose,
    pub lens: CameraLens,
    pub visibility_mask: u32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            name: String::new(),
            triggered: false,
            trigger_topic: String::new(),
            camera_info_topic: String::new(),
            horizontal_fov: 1.047,
            image_width: 320,
            image_height: 240,
            pixel_format: CameraPixelFormat::default(),
            anti_aliasing: 4,
            depth_near_clip: 0.1,
            has_depth_near_clip: false,
            depth_far_clip: 10.,
            has_depth_far_clip: false,
            near_clip: 0.1,
            far_clip: 100.,
            segmentation_type: "semantic".into(),
            has_segmentation_type: false,
            bounding_box_type: "2d".into(),
            has_bounding_box_type: false,
            save_frames: false,
            save_frames_path: String::new(),
            distortion: CameraDistortion::default(),
            pose: Pose::default(),
            lens: CameraLens::default(),
            visibility_mask: u32::MAX,
        }
    }
}
