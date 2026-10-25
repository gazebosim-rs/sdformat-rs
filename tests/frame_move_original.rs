/*
 * Copyright (C) 2017 Open Source Robotics Foundation
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
// Exact assertion port: DOMRoot.FrameSemanticsOnMove, src/Root_TEST.cc:243.

use sdformat::{NoResources, Sdf, Transform, load_str};

const SDF1: &str = r#"
    <sdf version="1.8">
      <world name="default">
        <frame name="frame1">
          <pose>0 1 0 0 0 0</pose>
        </frame>
      </world>
    </sdf>"#;

const SDF2: &str = r#"
    <sdf version="1.8">
      <world name="default">
        <frame name="frame2">
          <pose>1 1 0 0 0 0</pose>
        </frame>
      </world>
    </sdf>"#;

fn check_frame(sdf: &Sdf, name: &str, xyz: [f64; 3]) {
    let world = sdf.worlds.first().expect("original WorldByIndex(0)");
    assert_eq!(world.name, "default");
    let frame = world.frames.first().expect("original FrameByIndex(0)");
    assert_eq!(frame.name, name);
    let pose = sdf.pose_of(&format!("{}::{name}", world.name), &world.name);
    assert!(pose.is_ok(), "original SemanticPose.Resolve: {pose:?}");
    let actual = pose.unwrap();
    let expected = Transform {
        translation: xyz,
        rotation: [0., 0., 0., 1.],
    };
    for (actual, expected) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(expected.translation.into_iter().chain(expected.rotation))
    {
        assert!((actual - expected).abs() <= 1e-3);
    }
}

#[test]
fn original_frame_semantics_on_move() {
    {
        let root1 = load_str(SDF1, "frame1.sdf", &NoResources).unwrap();
        check_frame(&root1, "frame1", [0., 1., 0.]);
    }
    {
        let root2 = load_str(SDF2, "frame2.sdf", &NoResources).unwrap();
        check_frame(&root2, "frame2", [1., 1., 0.]);
    }
    {
        let root1 = load_str(SDF1, "frame1.sdf", &NoResources).unwrap();
        let root2 = root1;
        check_frame(&root2, "frame1", [0., 1., 0.]);
    }
    {
        let root1 = load_str(SDF1, "frame1.sdf", &NoResources).unwrap();
        let mut root2 = load_str(SDF2, "frame2.sdf", &NoResources).unwrap();
        check_frame(&root1, "frame1", [0., 1., 0.]);
        check_frame(&root2, "frame2", [1., 1., 0.]);
        root2 = root1;
        check_frame(&root2, "frame1", [0., 1., 0.]);
    }
}
