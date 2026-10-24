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
// Original assertion port: NestedReference.PlacementFrameAttribute in
// sdformat/test/integration/nested_model.cc, OSRF, Apache-2.0.
use sdformat::*;

fn load(xml: &str) -> Sdf {
    load_str(xml, "placement.sdf", &NoResources).unwrap()
}
fn close(actual: Transform, xyz: [f64; 3], rpy: [f64; 3]) {
    for (a, b) in actual.translation.into_iter().zip(xyz) {
        assert!((a - b).abs() < 1e-6, "{actual:?}");
    }
    let expected = glam::DQuat::from_euler(glam::EulerRot::ZYX, rpy[2], rpy[1], rpy[0]);
    assert!(glam::DQuat::from_array(actual.rotation).abs_diff_eq(expected, 1e-6));
}
#[test]
fn nested_reference_placement_frame_attribute() {
    let sdf = load(include_str!("fixtures/nested_placement_frame.sdf"));
    assert_eq!(sdf.models[0].name, "model_placement_frame");
    close(
        sdf.pose_of("model_placement_frame", "::").unwrap(),
        [0., -2., 10.],
        [0.; 3],
    );
    close(
        sdf.pose_of("model_placement_frame::child_model::link2", "::")
            .unwrap(),
        [0., 0., 10.],
        [0.; 3],
    );
    assert_eq!(sdf.models[0].pose.transform.translation, [0., 0., 10.]);
    assert_eq!(sdf.models[0].placement_frame, "child_model::link2");
}
struct ResourceFixture;
impl ResourceResolver for ResourceFixture {
    fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
        assert!(matches!(
            uri,
            "fixture" | "model_with_nested_placement_frame_attribute"
        ));
        Ok(Resource {
            canonical_uri: "memory:fixture".into(),
            xml: include_str!("fixtures/nested_placement_frame.sdf").into(),
        })
    }
}
fn included(override_xml: &str) -> Result<Sdf, Diagnostic> {
    load_str(
        &format!(
            "<sdf version='1.12'><world name='w'><include><uri>fixture</uri><name>m</name>{override_xml}</include></world></sdf>"
        ),
        "included.sdf",
        &ResourceFixture,
    )
}
#[test]
fn included_placement_overrides_preserve_source_and_require_pose() {
    let sdf = included("").unwrap();
    close(sdf.pose_of("w::m", "w").unwrap(), [0., -2., 10.], [0.; 3]);
    assert_eq!(sdf.worlds[0].models[0].source.uri, "memory:fixture");
    let sdf=included("<placement_frame>child_model::link3</placement_frame><pose>0 10 0 1.5707963267948966 0 0</pose>").unwrap();
    let r = std::f64::consts::FRAC_PI_2;
    close(
        sdf.pose_of("w::m", "w").unwrap(),
        [0., 13., 0.],
        [r, 0., 0.],
    );
    close(
        sdf.pose_of("w::m::child_model::link3", "w").unwrap(),
        [0., 10., 0.],
        [r, 0., 0.],
    );
    let sdf = included("<placement_frame/><pose>0 4 0 0 0 0</pose>").unwrap();
    assert_eq!(sdf.worlds[0].models[0].placement_frame, "");
    close(sdf.pose_of("w::m", "w").unwrap(), [0., 4., 0.], [0.; 3]);
    for value in [
        "<placement_frame>child_model::link3</placement_frame>",
        "<placement_frame/>",
    ] {
        assert_eq!(included(value).unwrap_err().code, ErrorCode::Missing);
    }
    assert_eq!(
        included("<placement_frame>missing</placement_frame><pose/>")
            .unwrap_err()
            .code,
        ErrorCode::UnknownReference
    );
}
#[test]
fn frames_links_joints_and_nested_model_frames_can_place_a_model() {
    for target in ["l", "f", "j", "nested", "nested::l", "__model__"] {
        let xml = format!(
            "<sdf version='1.12'><model name='m' placement_frame='{target}'><pose>3 4 5 0 0 0</pose><link name='base'/><link name='l'><pose>1 2 0 0 0 0</pose></link><frame name='f' attached_to='l'><pose>0 0 1 0 0 0</pose></frame><joint name='j' type='fixed'><parent>base</parent><child>l</child></joint><model name='nested' placement_frame='l'><pose>0 1 0 0 0 0</pose><link name='l'><pose>0 0 2 0 0 0</pose></link></model></model></sdf>"
        );
        let sdf = load(&xml);
        let name = if target == "__model__" {
            "m".into()
        } else {
            format!("m::{target}")
        };
        close(sdf.pose_of(&name, "::").unwrap(), [3., 4., 5.], [0.; 3]);
    }
    let xml = "<sdf version='1.12'><world name='w'><frame name='origin'><pose>10 0 0 0 0 0</pose></frame><model name='m' placement_frame='l'><pose relative_to='origin'>0 2 0 0 0 0</pose><link name='l'><pose>0 0 1 0 0 0</pose></link></model></world></sdf>";
    let sdf = load(xml);
    close(sdf.pose_of("w::m::l", "w").unwrap(), [10., 2., 0.], [0.; 3]);
    close(sdf.pose_of("w::m", "w").unwrap(), [10., 2., -1.], [0.; 3]);
    let xml = "<sdf version='1.12'><model name='m' placement_frame='missing'><link name='l'/></model></sdf>";
    assert_eq!(
        load_str(xml, "missing", &NoResources).unwrap_err().code,
        ErrorCode::UnknownReference
    );
}
#[test]
fn placement_is_applied_before_absolute_pose_overflow_validation() {
    let sdf = load(
        "<sdf version='1.12'><model name='m' placement_frame='l'><pose>1e308 0 0 0 0 0</pose><link name='l'><pose>1e308 0 0 0 0 0</pose></link></model></sdf>",
    );
    assert_eq!(sdf.pose_of("m", "::").unwrap().translation, [0.; 3]);
    assert_eq!(
        sdf.pose_of("m::l", "::").unwrap().translation,
        [1e308, 0., 0.]
    );
}

#[test]
fn nested_reference_placement_frame_element_all_four_original_cases() {
    let uri = "model_with_nested_placement_frame_attribute";
    let default = format!("<include><uri>{uri}</uri></include>");
    let overridden = format!(
        "<include><uri>{uri}</uri><placement_frame>child_model::link3</placement_frame><pose>0 10 0 1.5707963267948966 0 0</pose>\"</include>"
    );
    let r = std::f64::consts::FRAC_PI_2;
    // world/include with the imported model's placement_frame.
    let sdf = load_str(
        &format!("<sdf version='1.8'><world name='default'>{default}</world></sdf>"),
        "world-include",
        &ResourceFixture,
    )
    .unwrap();
    let world = &sdf.worlds[0];
    let model = &world.models[0];
    close(
        sdf.pose_of(&model.scoped_name, &world.name).unwrap(),
        [0., -2., 10.],
        [0.; 3],
    );
    // world/include overrides it and resolves from the original world frame.
    let sdf=load_str(&format!("<sdf version='1.8'><world name='default'><frame name='world_frame'/>{overridden}</world></sdf>"),"world-override",&ResourceFixture).unwrap();
    assert!(
        sdf.worlds[0]
            .frames
            .iter()
            .any(|frame| frame.name == "world_frame")
    );
    close(
        sdf.pose_of(
            "default::model_placement_frame::child_model::link3",
            "default::world_frame",
        )
        .unwrap(),
        [0., 10., 0.],
        [r, 0., 0.],
    );
    // model/include retains the imported model's placement_frame.
    let sdf = load_str(
        &format!("<sdf version='1.8'><model name='parent_model'>{default}</model></sdf>"),
        "model-include",
        &ResourceFixture,
    )
    .unwrap();
    let parent = &sdf.models[0];
    let model = &parent.models[0];
    close(
        sdf.pose_of(&model.scoped_name, &parent.scoped_name)
            .unwrap(),
        [0., -2., 10.],
        [0.; 3],
    );
    // model/include overrides it and retains nested link lookup assertions.
    let sdf=load_str(&format!("<sdf version='1.8'><model name='parent_model'><frame name='parent_model_frame'/>{overridden}</model></sdf>"),"model-override",&ResourceFixture).unwrap();
    let parent = &sdf.models[0];
    let model = &parent.models[0];
    assert!(
        parent
            .frames
            .iter()
            .any(|frame| frame.name == "parent_model_frame")
    );
    assert!(
        model.models[0]
            .links
            .iter()
            .any(|link| link.name == "link3")
    );
    close(
        sdf.pose_of(
            "parent_model::model_placement_frame::child_model::link3",
            "parent_model::parent_model_frame",
        )
        .unwrap(),
        [0., 10., 0.],
        [r, 0., 0.],
    );
}

#[test]
fn corrected_model_pose_overflow_is_reported() {
    let xml = "<sdf version='1.12'><model name='m' placement_frame='l'><pose>1e308 0 0 0 0 0</pose><link name='l'><pose>-1e308 0 0 0 0 0</pose></link></model></sdf>";
    assert_eq!(
        load_str(xml, "placement-overflow", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
}
