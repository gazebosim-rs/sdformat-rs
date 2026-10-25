/*
 * Copyright (C) 2019 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

use sdformat::{Camera, CameraPixelFormat};

#[test]
fn original_camera_construction_values_copy_and_move() {
    let mut cam = Camera::default();
    assert!(cam.name.is_empty());
    cam.name = "my_camera".into();
    assert_eq!(cam.name, "my_camera");
    assert!(!cam.triggered);
    cam.triggered = true;
    assert!(cam.triggered);
    assert!(cam.trigger_topic.is_empty());
    cam.trigger_topic = "my_camera/trigger".into();
    assert_eq!(cam.trigger_topic, "my_camera/trigger");
    assert_eq!(cam.camera_info_topic, "");
    cam.camera_info_topic = "/camera/camera_info".into();
    assert_eq!(cam.camera_info_topic, "/camera/camera_info");
    assert_eq!(cam.horizontal_fov, 1.047);
    cam.horizontal_fov = 1.45;
    assert_eq!(cam.horizontal_fov, 1.45);
    assert_eq!(cam.image_width, 320);
    cam.image_width = 123;
    assert_eq!(cam.image_width, 123);
    assert_eq!(cam.image_height, 240);
    cam.image_height = 125;
    assert_eq!(cam.image_height, 125);
    assert_eq!(cam.pixel_format, CameraPixelFormat::RgbInt8);
    cam.pixel_format = CameraPixelFormat::LInt8;
    assert_eq!(cam.pixel_format, CameraPixelFormat::LInt8);
    assert_eq!(cam.anti_aliasing, 4);
    cam.anti_aliasing = 8;
    assert_eq!(cam.anti_aliasing, 8);

    assert_eq!(cam.depth_near_clip, 0.1);
    assert!(!cam.has_depth_near_clip);
    cam.depth_near_clip = 0.2;
    cam.has_depth_near_clip = true;
    assert_eq!(cam.depth_near_clip, 0.2);
    assert!(cam.has_depth_near_clip);
    assert_eq!(cam.depth_far_clip, 10.);
    assert!(!cam.has_depth_far_clip);
    cam.depth_far_clip = 20.2;
    cam.has_depth_far_clip = true;
    assert_eq!(cam.depth_far_clip, 20.2);
    assert!(cam.has_depth_far_clip);
    assert_eq!(cam.near_clip, 0.1);
    cam.near_clip = 0.2;
    assert_eq!(cam.near_clip, 0.2);
    assert_eq!(cam.far_clip, 100.);
    cam.far_clip = 200.2;
    assert_eq!(cam.far_clip, 200.2);
    assert_eq!(cam.segmentation_type, "semantic");
    assert!(!cam.has_segmentation_type);
    cam.segmentation_type = "panoptic".into();
    cam.has_segmentation_type = true;
    assert!(cam.has_segmentation_type);
    assert_eq!(cam.segmentation_type, "panoptic");
    cam.has_segmentation_type = false;
    assert!(!cam.has_segmentation_type);
    assert_eq!(cam.bounding_box_type, "2d");
    assert!(!cam.has_bounding_box_type);
    cam.bounding_box_type = "3d".into();
    cam.has_bounding_box_type = true;
    assert!(cam.has_bounding_box_type);
    assert_eq!(cam.bounding_box_type, "3d");
    cam.has_bounding_box_type = false;
    assert!(!cam.has_bounding_box_type);
    assert!(!cam.save_frames);
    cam.save_frames = true;
    assert!(cam.save_frames);
    assert_eq!(cam.save_frames_path, "");
    cam.save_frames_path = "/tmp".into();
    assert_eq!(cam.save_frames_path, "/tmp");

    assert_eq!(cam.distortion.k1, 0.);
    cam.distortion.k1 = 0.1;
    assert_eq!(cam.distortion.k1, 0.1);
    assert_eq!(cam.distortion.k2, 0.);
    cam.distortion.k2 = 0.2;
    assert_eq!(cam.distortion.k2, 0.2);
    assert_eq!(cam.distortion.k3, 0.);
    cam.distortion.k3 = 0.2;
    assert_eq!(cam.distortion.k3, 0.2);
    assert_eq!(cam.distortion.p1, 0.);
    cam.distortion.p1 = 0.2;
    assert_eq!(cam.distortion.p1, 0.2);
    assert_eq!(cam.distortion.p2, 0.);
    cam.distortion.p2 = 0.2;
    assert_eq!(cam.distortion.p2, 0.2);
    assert_eq!(cam.distortion.center, [0.5, 0.5]);
    cam.distortion.center = [0.1, 0.2];
    assert_eq!(cam.distortion.center, [0.1, 0.2]);

    assert_eq!(cam.pose.transform.translation, [0.; 3]);
    assert_eq!(cam.pose.transform.rotation, [0., 0., 0., 1.]);
    cam.pose.transform.translation = [1., 2., 3.];
    assert_eq!(cam.pose.transform.translation, [1., 2., 3.]);
    assert_eq!(cam.pose.transform.rotation, [0., 0., 0., 1.]);
    assert!(cam.pose.relative_to.is_empty());
    cam.pose.relative_to = "/frame".into();
    assert_eq!(cam.pose.relative_to, "/frame");

    assert_eq!(cam.lens.kind, "stereographic");
    cam.lens.kind = "custom".into();
    assert_eq!(cam.lens.kind, "custom");
    assert!(cam.lens.scale_to_hfov);
    cam.lens.scale_to_hfov = false;
    assert!(!cam.lens.scale_to_hfov);
    assert_eq!(cam.lens.c1, 1.);
    cam.lens.c1 = 2.1;
    assert_eq!(cam.lens.c1, 2.1);
    assert_eq!(cam.lens.c2, 1.);
    cam.lens.c2 = 1.2;
    assert_eq!(cam.lens.c2, 1.2);
    assert_eq!(cam.lens.c3, 0.);
    cam.lens.c3 = 6.5;
    assert_eq!(cam.lens.c3, 6.5);
    assert_eq!(cam.lens.focal_length, 1.);
    cam.lens.focal_length = 10.3;
    assert_eq!(cam.lens.focal_length, 10.3);
    assert_eq!(cam.lens.function, "tan");
    cam.lens.function = "sin".into();
    assert_eq!(cam.lens.function, "sin");
    assert_eq!(cam.lens.cutoff_angle, std::f64::consts::FRAC_PI_2);
    cam.lens.cutoff_angle = 0.456;
    assert_eq!(cam.lens.cutoff_angle, 0.456);
    assert_eq!(cam.lens.environment_texture_size, 256);
    cam.lens.environment_texture_size = 512;
    assert_eq!(cam.lens.environment_texture_size, 512);

    assert_eq!(cam.lens.intrinsics.fx, 277.);
    cam.lens.intrinsics.fx = 132.;
    assert_eq!(cam.lens.intrinsics.fx, 132.);
    assert_eq!(cam.lens.intrinsics.fy, 277.);
    cam.lens.intrinsics.fy = 456.;
    assert_eq!(cam.lens.intrinsics.fy, 456.);
    assert_eq!(cam.lens.intrinsics.cx, 160.);
    cam.lens.intrinsics.cx = 254.;
    assert_eq!(cam.lens.intrinsics.cx, 254.);
    assert_eq!(cam.lens.intrinsics.cy, 120.);
    cam.lens.intrinsics.cy = 123.;
    assert_eq!(cam.lens.intrinsics.cy, 123.);
    assert_eq!(cam.lens.projection.fx, 277.);
    cam.lens.projection.fx = 132.;
    assert_eq!(cam.lens.projection.fx, 132.);
    assert_eq!(cam.lens.projection.fy, 277.);
    cam.lens.projection.fy = 456.;
    assert_eq!(cam.lens.projection.fy, 456.);
    assert_eq!(cam.lens.projection.cx, 160.);
    cam.lens.projection.cx = 254.;
    assert_eq!(cam.lens.projection.cx, 254.);
    assert_eq!(cam.lens.projection.cy, 120.);
    cam.lens.projection.cy = 123.;
    assert_eq!(cam.lens.projection.cy, 123.);
    assert_eq!(cam.lens.projection.tx, 0.);
    cam.lens.projection.tx = 1.;
    assert_eq!(cam.lens.projection.tx, 1.);
    assert_eq!(cam.lens.projection.ty, 0.);
    cam.lens.projection.ty = 2.;
    assert_eq!(cam.lens.projection.ty, 2.);
    assert_eq!(cam.lens.intrinsics.skew, 0.);
    cam.lens.intrinsics.skew = 2.3;
    assert_eq!(cam.lens.intrinsics.skew, 2.3);
    assert!(cam.lens.has_intrinsics);
    assert!(cam.lens.has_projection);
    assert_eq!(cam.visibility_mask, u32::MAX);
    cam.visibility_mask = 123;
    assert_eq!(cam.visibility_mask, 123);

    let mut copied = cam.clone();
    assert_eq!(cam, copied, "copy construction");
    let mut assigned = Camera::default();
    assigned.clone_from(&cam);
    assert_eq!(cam, assigned, "copy assignment");
    let moved = cam;
    assert_eq!(copied, moved, "move construction");
    cam = moved.clone();
    assert_eq!(copied, cam, "copy after move");
    let mut move_assigned = Camera::default();
    assert_eq!(move_assigned, Camera::default());
    move_assigned = copied;
    assert_eq!(assigned, move_assigned, "move assignment");
    copied = move_assigned.clone();
    assert_eq!(assigned, copied, "copy after move assignment");
    assert_ne!(assigned, Camera::default(), "default inequality");
}
