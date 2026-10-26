/*
 * Copyright 2018 Open Source Robotics Foundation
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
// Fixed-fixture observations from DOMJointAxis.Complete.
use sdformat::{JointKind, NoResources, load_str};

#[test]
fn original_dom_joint_axis_complete() {
    let sdf = load_str(
        include_str!("fixtures/joint_complete.sdf"),
        "joint_complete.sdf",
        &NoResources,
    )
    .expect("native Root.Load has no errors");
    let model = sdf.models.first().expect("native Root.Model is present");
    assert_eq!(model.joints.len(), 9);
    let kinds = [
        JointKind::Revolute,
        JointKind::Ball,
        JointKind::Continuous,
        JointKind::Fixed,
        JointKind::Gearbox,
        JointKind::Prismatic,
        JointKind::Revolute2,
        JointKind::Screw,
        JointKind::Universal,
    ];
    for (index, kind) in kinds.into_iter().enumerate() {
        assert_eq!(model.joints.get(index).expect("JointByIndex").kind, kind);
    }
    // Source-backed controls for known fixture fields outside this original's
    // assertions: retain raw physics and convert legacy turns-per-metre pitch.
    assert_eq!(
        model.joints[7].screw_thread_pitch,
        -std::f64::consts::PI / 10.
    );
    assert_eq!(model.joints[1].screw_thread_pitch, 1.);
    let joint = model.joints.first().expect("JointByIndex(0)");
    assert_eq!(joint.kind, JointKind::Revolute);
    assert_eq!(joint.source.uri, "joint_complete.sdf");
    let physics = joint
        .physics_xml
        .as_deref()
        .expect("known physics XML retained");
    assert!(physics.contains("<cfm_damping>1</cfm_damping>"));
    assert!(physics.contains("<erp>0.2</erp>"));
    let axis = joint.axis(0).expect("Axis(0) from fixture XML");
    let axis2 = joint.axis(1).expect("Axis(1) from fixture XML");
    assert_eq!(axis.xyz, [0., 0., 1.]);
    assert_eq!(axis2.xyz, [0., 1., 0.]);
    assert_eq!(axis.expressed_in, "__model__");
    assert!(axis2.expressed_in.is_empty());
    assert_eq!((axis.lower, axis.upper), (-0.5, 0.5));
    assert_eq!((axis2.lower, axis2.upper), (-1., 1.));
    assert_eq!((axis.effort, axis2.effort), (123.4, 0.5));
    assert_eq!((axis.velocity, axis2.velocity), (12., 200.));
    assert_eq!((axis.dynamics.damping, axis2.dynamics.damping), (0.1, 0.));
    assert_eq!((axis.dynamics.friction, axis2.dynamics.friction), (0.2, 0.));
    assert_eq!(
        (
            axis.dynamics.spring_reference,
            axis2.dynamics.spring_reference
        ),
        (1.3, 0.)
    );
    assert_eq!(
        (
            axis.dynamics.spring_stiffness,
            axis2.dynamics.spring_stiffness
        ),
        (10.6, 0.)
    );
}

#[test]
fn native_screw_pitch_precedence_is_retained() {
    let xml = "<sdf version='1.12'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='screw'><parent>a</parent><child>b</child><thread_pitch>20</thread_pitch><screw_thread_pitch>1.25</screw_thread_pitch><axis><xyz>0 0 1</xyz></axis></joint></model></sdf>";
    let sdf = load_str(xml, "pitch.sdf", &NoResources).unwrap();
    assert_eq!(sdf.models[0].joints[0].screw_thread_pitch, 1.25);
}
