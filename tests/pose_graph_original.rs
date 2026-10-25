/*
 * Copyright 2019 Open Source Robotics Foundation
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
//
// Partial assertion port: FrameSemantics.buildPoseRelativeToGraph,
// sdformat/src/FrameSemantics_TEST.cc. The fixture is copied byte-for-byte.

use glam::DQuat;
use sdformat::{ErrorCode, NoResources, Transform, load_str};
use std::f64::consts::FRAC_PI_2;

fn expected(x: f64, y: f64, z: f64, pitch: f64) -> Transform {
    Transform {
        translation: [x, y, z],
        rotation: DQuat::from_rotation_y(pitch).to_array(),
    }
}

fn close(actual: Transform, expected: Transform, name: &str) {
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected.translation.into_iter().chain(expected.rotation))
    {
        assert!((a - b).abs() <= 1e-3, "{name}: {actual:?} vs {expected:?}");
    }
}

#[test]
fn original_pose_relative_to_graph_public_queries() {
    let sdf = load_str(
        include_str!("fixtures/model_frame_relative_to_joint.sdf"),
        "model_frame_relative_to_joint.sdf",
        &NoResources,
    )
    .unwrap();
    let scope = &sdf.models[0].scoped_name;

    // The native original resolves each frame against the model graph root.
    for (name, pose) in [
        ("__model__", expected(0., 0., 0., 0.)),
        ("P", expected(1., 0., 0., 0.)),
        ("F1", expected(1., 0., 1., 0.)),
        ("C", expected(2., 0., 0., FRAC_PI_2)),
        ("F2", expected(4., 0., 0., FRAC_PI_2)),
        ("J", expected(2., 3., 0., 0.)),
        ("F3", expected(2., 3., 3., FRAC_PI_2)),
        ("F4", expected(6., 3., 3., 0.)),
    ] {
        let frame = if name == "__model__" {
            scope.clone()
        } else {
            format!("{scope}::{name}")
        };
        close(sdf.pose_of(&frame, scope).unwrap(), pose, name);
    }

    // These are the original's source/target queries using each raw pose's
    // relative_to frame; the expected values are its exact input pose values.
    for (name, relative_to, pose) in [
        ("P", "__model__", expected(1., 0., 0., 0.)),
        ("C", "__model__", expected(2., 0., 0., FRAC_PI_2)),
        ("J", "C", expected(0., 3., 0., -FRAC_PI_2)),
        ("F1", "P", expected(0., 0., 1., 0.)),
        ("F2", "C", expected(0., 0., 2., 0.)),
        ("F3", "J", expected(0., 0., 3., FRAC_PI_2)),
        ("F4", "F3", expected(0., 0., 4., -FRAC_PI_2)),
    ] {
        let target = if relative_to == "__model__" {
            scope.clone()
        } else {
            format!("{scope}::{relative_to}")
        };
        close(
            sdf.pose_of(&format!("{scope}::{name}"), &target).unwrap(),
            pose,
            name,
        );
    }

    for (source, target) in [("invalid", scope.as_str()), (scope.as_str(), "invalid")] {
        let err = sdf.pose_of(source, target).unwrap_err();
        assert_eq!(err.code, ErrorCode::UnknownReference);
    }
}
