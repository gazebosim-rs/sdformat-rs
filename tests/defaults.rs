// SPDX-License-Identifier: Apache-2.0
// Copyright 2015 Open Source Robotics Foundation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Original nested-frame assertions: sdformat/test/integration/nested_model.cc
// NestedModel.NestedFrameOnlyModel, OSRF, Apache-2.0.
use sdformat::*;
#[test]
fn moving_joint_axis_absence_is_preserved_for_backend_capability_checks() {
    for kind in ["fixed", "revolute", "prismatic", "continuous"] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'/><link name='r'/><joint name='j' type='{kind}'><parent>l</parent><child>r</child></joint></model></sdf>"
        );
        let sdf = load_str(&xml, "optional-axis", &NoResources).unwrap();
        let joint = &sdf.models[0].joints[0];
        assert_eq!(joint.name, "j");
        assert_eq!(joint.parent, "l");
        assert_eq!(joint.child, "r");
        assert!(joint.axis.is_none(), "{kind}");
    }
}
struct FrameOnly;
impl ResourceResolver for FrameOnly {
    fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
        assert_eq!(uri, "frame_only_model");
        Ok(Resource {
            canonical_uri: "fixture:frame_only_model".into(),
            xml: include_str!("fixtures/frame_only_model.sdf").into(),
        })
    }
}
#[test]
fn nested_frame_only_model_original_pose_assertions() {
    let xml = "<sdf version='1.8'><world name='default'><model name='ParentModel'><static>true</static><include><uri>frame_only_model</uri><name>M1</name><pose>10 0 0 0 0 1.5707963267948966</pose></include></model></world></sdf>";
    let sdf = load_str(xml, "nested-frame-only", &FrameOnly).unwrap();
    let world = &sdf.worlds[0];
    let parent = &world.models[0];
    let yaw = glam::DQuat::from_rotation_z(std::f64::consts::FRAC_PI_2);
    let frame1 = yaw * glam::DQuat::from_rotation_x(std::f64::consts::PI);
    let frame2 = frame1 * glam::DQuat::from_rotation_y(std::f64::consts::PI);
    for (frame, expected) in [("F1", frame1), ("F2", frame2)] {
        let frame_name = format!("default::ParentModel::M1::{frame}");
        let resolved = sdf.pose_of(&parent.scoped_name, &frame_name).unwrap();
        let rotation = glam::DQuat::from_array(resolved.rotation).conjugate();
        let actual = Transform {
            translation: (rotation * -glam::DVec3::from_array(resolved.translation)).to_array(),
            rotation: rotation.to_array(),
        };
        // The original resolves parent in the nested frame and then inverts.
        let forward = sdf.pose_of(&frame_name, &parent.scoped_name).unwrap();
        for (a, b) in forward.translation.into_iter().zip(actual.translation) {
            assert!((a - b).abs() < 1e-12);
        }
        for (a, b) in actual.translation.into_iter().zip([10., 0., 0.]) {
            assert!((a - b).abs() < 1e-6);
        }
        assert!(
            glam::DQuat::from_array(actual.rotation).abs_diff_eq(expected, 1e-6),
            "{actual:?}"
        );
    }
    assert!(parent.models[0].is_static);
    assert_eq!(parent.models[0].source.uri, "fixture:frame_only_model");
}
#[test]
fn sdf17_subset_preserves_version_and_reports_unimplemented_unflattening() {
    let xml = "<sdf version='1.7'><model name='m'><link name='l'><inertial><pose relative_to=''>0 0 0 0 0 0</pose></inertial></link></model></sdf>";
    let sdf = load_str(xml, "1.7", &NoResources).unwrap();
    assert_eq!(sdf.version, "1.7");
    assert_eq!(sdf.models[0].links[0].inertial.pose.relative_to, "");
    let model = &sdf.models[0];
    let link = &model.links[0];
    assert_eq!(
        sdf.resolve_pose(
            &link.inertial.pose,
            &model.scoped_name,
            &link.scoped_name,
            &link.scoped_name
        )
        .unwrap(),
        Transform::default()
    );
    let flat = "<sdf version='1.7'><model name='m'><link name='nested::l'/></model></sdf>";
    assert_eq!(
        load_str(flat, "unflatten", &NoResources).unwrap_err().code,
        ErrorCode::Unsupported
    );
    let older = "<sdf version='1.6'><model name='m'><link name='l'/></model></sdf>";
    assert_eq!(
        load_str(older, "older", &NoResources).unwrap_err().code,
        ErrorCode::Version
    );
}
