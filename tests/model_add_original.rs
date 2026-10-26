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
// Observable values from DOMModel.AddJoint and DOMModel.AddModel.
use sdformat::{Joint, Model, NoResources, load_str};

fn empty_model() -> Model {
    load_str(
        "<sdf version='1.12'><model name='parent'><static>true</static></model></sdf>",
        "model-add.sdf",
        &NoResources,
    )
    .unwrap()
    .models
    .remove(0)
}

fn joint() -> Joint {
    load_str("<sdf version='1.12'><model name='parent'><link name='a'/><link name='b'/><joint name='joint1' type='fixed'><parent>a</parent><child>b</child></joint></model></sdf>", "joint-add.sdf", &NoResources)
        .unwrap()
        .models[0]
        .joints
        .remove(0)
}

fn nested_model() -> Model {
    load_str("<sdf version='1.12'><model name='parent'><model name='model1'><link name='base'/></model></model></sdf>", "nested-add.sdf", &NoResources)
        .unwrap()
        .models[0]
        .models
        .remove(0)
}

#[test]
fn original_dom_model_add_joint() {
    let mut model = empty_model();
    assert_eq!(model.joints.len(), 0);
    let joint = joint();
    assert!(model.add_joint_unique(joint.clone()));
    assert_eq!(model.joints.len(), 1);
    assert!(!model.add_joint_unique(joint.clone()));
    assert_eq!(model.joints.len(), 1);
    model.clear_joints();
    assert_eq!(model.joints.len(), 0);
    assert!(model.add_joint_unique(joint));
    assert_eq!(model.joints.len(), 1);
    let added = model.joints.first().expect("JointByIndex(0)");
    assert_eq!(added.name, "joint1");
}

#[test]
fn original_dom_model_add_model() {
    let mut model = empty_model();
    assert_eq!(model.models.len(), 0);
    let nested = nested_model();
    assert!(model.add_model_unique(nested));
    assert_eq!(model.models.len(), 1);
    assert!(!model.add_model_unique(nested_model()));
    assert_eq!(model.models.len(), 1);
    model.clear_models();
    assert_eq!(model.models.len(), 0);
    assert!(model.add_model_unique(nested_model()));
    assert_eq!(model.models.len(), 1);
    let added = model.models.first().expect("ModelByIndex(0)");
    assert_eq!(added.name, "model1");
}
