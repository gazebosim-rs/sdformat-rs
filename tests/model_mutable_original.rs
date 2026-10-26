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
// Observable member rename/relookup values from two DOMModel originals.
use sdformat::{Model, NoResources, load_str};

fn parsed_model() -> Model {
    let xml = "<sdf version='1.12'><model name='model1'><link name='link1'/><link name='aux'/><joint name='joint1' type='fixed'><parent>aux</parent><child>link1</child></joint><model name='child1'><link name='base'/></model><frame name='frame1' attached_to='link1'/></model></sdf>";
    load_str(xml, "model-mutable.sdf", &NoResources)
        .unwrap()
        .models
        .remove(0)
}

fn model(include_frame: bool) -> Model {
    let mut model = parsed_model();
    let mut duplicate = parsed_model();
    let mut links = std::mem::take(&mut model.links);
    let link = links.remove(0);
    let auxiliary = links.remove(0);
    let joint = model.joints.remove(0);
    let nested = model.models.remove(0);
    let frame = model.frames.remove(0);
    assert_eq!(model.models.len(), 0);
    assert!(model.add_link_unique(link));
    assert!(!model.add_link_unique(duplicate.links.remove(0)));
    assert!(model.add_link_unique(auxiliary));
    assert!(model.add_joint_unique(joint));
    assert!(model.add_model_unique(nested));
    if include_frame {
        assert!(model.add_frame_unique(frame));
        assert!(!model.add_frame_unique(duplicate.frames.remove(0)));
    }
    model
}

#[test]
fn original_dom_model_mutable_by_index() {
    let mut model = model(false);
    let link = model.links.get_mut(0).expect("LinkByIndex(0)");
    assert_eq!(link.name, "link1");
    link.name = "link2".into();
    assert_eq!(model.links.first().unwrap().name, "link2");

    let joint = model.joints.get_mut(0).expect("JointByIndex(0)");
    assert_eq!(joint.name, "joint1");
    joint.name = "joint2".into();
    assert_eq!(model.joints.first().unwrap().name, "joint2");

    let nested = model.models.get_mut(0).expect("ModelByIndex(0)");
    assert_eq!(nested.name, "child1");
    nested.name = "child2".into();
    assert_eq!(model.models.first().unwrap().name, "child2");
}

#[test]
fn original_dom_model_mutable_by_name() {
    let mut model = model(true);
    let link = model
        .links
        .iter_mut()
        .find(|member| member.name == "link1")
        .expect("LinkByName(link1)");
    assert_eq!(link.name, "link1");
    link.name = "link2".into();
    assert!(model.link_by_name("link1").is_none());
    assert!(model.link_by_name("link2").is_some());

    let joint = model
        .joints
        .iter_mut()
        .find(|member| member.name == "joint1")
        .expect("JointByName(joint1)");
    assert_eq!(joint.name, "joint1");
    joint.name = "joint2".into();
    assert!(model.joint_by_name("joint1").is_none());
    assert!(model.joint_by_name("joint2").is_some());

    let nested = model
        .models
        .iter_mut()
        .find(|member| member.name == "child1")
        .expect("ModelByName(child1)");
    assert_eq!(nested.name, "child1");
    nested.name = "child2".into();
    assert!(model.model_by_name("child1").is_none());
    assert!(model.model_by_name("child2").is_some());

    let frame = model
        .frames
        .iter_mut()
        .find(|member| member.name == "frame1")
        .expect("FrameByName(frame1)");
    assert_eq!(frame.name, "frame1");
    frame.name = "frame2".into();
    assert!(model.frame_by_name("frame1").is_none());
    assert!(model.frame_by_name("frame2").is_some());
}
