/*
 * Copyright (C) 2018 Open Source Robotics Foundation
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
// Observable values from DOMModel.AddModifyFrame.
use sdformat::{Frame, Model, NoResources, load_str};

fn model_and_frame() -> (Model, Frame) {
    let mut model = load_str(
        "<sdf version='1.12'><model name='parent'><static>true</static><frame name='frame1'/></model></sdf>",
        "model-frame.sdf",
        &NoResources,
    )
    .unwrap()
    .models
    .remove(0);
    let frame = model.frames.remove(0);
    (model, frame)
}

#[test]
fn original_dom_model_add_modify_frame() {
    let (mut model, frame) = model_and_frame();
    assert_eq!(model.frames.len(), 0);

    assert!(model.add_frame_unique(frame));
    assert_eq!(model.frames.len(), 1);
    let (_, duplicate) = model_and_frame();
    assert!(!model.add_frame_unique(duplicate));
    assert_eq!(model.frames.len(), 1);

    model.clear_frames();
    assert_eq!(model.frames.len(), 0);
    let (_, frame) = model_and_frame();
    assert!(model.add_frame_unique(frame));
    assert_eq!(model.frames.len(), 1);

    let frame_from_model = model.frames.first().expect("FrameByIndex(0)");
    assert_eq!(frame_from_model.name, "frame1");
    model
        .frames
        .first_mut()
        .expect("mutable FrameByIndex(0)")
        .name = "newName1".into();
    assert_eq!(model.frames.first().unwrap().name, "newName1");
    assert!(model.frame_by_name("frame1").is_none());
    assert_eq!(model.frame_by_name("newName1").unwrap().name, "newName1");
}
