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
use sdformat::*;
fn pose(actual: Transform, xyz: [f64; 3]) {
    for (a, b) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(xyz.into_iter().chain([0., 0., 0., 1.]))
    {
        assert!((a - b).abs() <= 1e-3, "pose {a} != {b}");
    }
}
fn double(actual: f64, expected: f64) {
    assert!(actual.is_finite() && expected.is_finite() && actual >= 0. && expected >= 0.);
    assert!(
        actual.to_bits().abs_diff(expected.to_bits()) <= 4,
        "{actual} != {expected}"
    );
}
#[test]
fn original_dom_world_load_lights() {
    let sdf = load_str(
        include_str!("fixtures/light_documents_original.sdf"),
        "world_complete.sdf#lights",
        &NoResources,
    )
    .unwrap();
    let world = sdf.worlds.first().expect("native WorldByIndex0 presence");
    let point = world
        .lights
        .first()
        .expect("native point LightByIndex0 presence");
    assert_eq!(point.name, "point_light");
    assert_eq!(point.light_type, LightType::Point);
    assert!(!point.cast_shadows);
    pose(point.pose.transform, [1., 2., 10.]);
    assert_eq!(point.pose.relative_to, "world");
    pose(
        sdf.resolve_pose(&point.pose, &world.name, &world.name, &world.name)
            .unwrap(),
        [1., 2., 10.],
    );
    pose(
        sdf.resolve_pose(&point.pose, &world.name, &world.name, &world.name)
            .unwrap(),
        [1., 2., 10.],
    );
    double(point.attenuation_range, 123.5);
    double(point.linear_attenuation_factor, 1.);
    double(point.constant_attenuation_factor, 0.);
    double(point.quadratic_attenuation_factor, 20.2);
    double(point.intensity, 1.);
    let spot = world
        .lights
        .get(1)
        .expect("native spot LightByIndex1 presence");
    assert_eq!(spot.name, "spot_light");
    assert_eq!(spot.light_type, LightType::Spot);
    assert!(spot.cast_shadows);
    double(spot.spot_inner_angle, 0.1);
    double(spot.spot_outer_angle, 0.5);
    double(spot.spot_falloff, 2.2);
    double(spot.intensity, 1.);
    let directional = world
        .lights
        .get(2)
        .expect("native directional LightByIndex2 presence");
    assert_eq!(directional.name, "directional_light");
    pose(directional.pose.transform, [0., 10., 20.]);
    assert_eq!(directional.pose.relative_to, "frame1");
    let frame1 = format!("{}::frame1", world.name);
    pose(
        sdf.resolve_pose(&directional.pose, &world.name, &world.name, &frame1)
            .unwrap(),
        [0., 10., 20.],
    );
    pose(
        sdf.resolve_pose(&directional.pose, &world.name, &world.name, &world.name)
            .unwrap(),
        [1., 12., 23.],
    );
    pose(
        sdf.resolve_pose(&directional.pose, &world.name, &world.name, &world.name)
            .unwrap(),
        [1., 12., 23.],
    );
    double(directional.attenuation_range, 0.);
    double(directional.linear_attenuation_factor, 0.);
    double(directional.constant_attenuation_factor, 1.);
    double(directional.quadratic_attenuation_factor, 0.);
    double(directional.intensity, 1.8);
    let model = world.models.first().expect("native ModelByIndex0 presence");
    pose(model.pose.transform, [0.; 3]);
    assert_eq!(model.pose.relative_to, "frame1");
    sdf.pose_of(&model.scoped_name, &frame1).unwrap();
    pose(sdf.pose_of(&model.scoped_name, &frame1).unwrap(), [0.; 3]);
    pose(
        sdf.pose_of(&model.scoped_name, &world.name).unwrap(),
        [1., 2., 3.],
    );
    pose(
        sdf.pose_of(&model.scoped_name, &world.name).unwrap(),
        [1., 2., 3.],
    );
    let link = model.links.first().expect("native LinkByIndex0 presence");
    let light = link
        .lights
        .first()
        .expect("native Link LightByIndex0 presence");
    assert_eq!(light.name, "spot_light");
    let frame2 = format!("{}::frame2", model.scoped_name);
    pose(
        sdf.resolve_pose(&light.pose, &model.scoped_name, &link.scoped_name, &frame2)
            .unwrap(),
        [7., 8., 9.],
    );
    pose(
        sdf.resolve_pose(
            &light.pose,
            &model.scoped_name,
            &link.scoped_name,
            &model.scoped_name,
        )
        .unwrap(),
        [11., 13., 15.],
    );
    pose(
        sdf.resolve_pose(
            &light.pose,
            &model.scoped_name,
            &link.scoped_name,
            &link.scoped_name,
        )
        .unwrap(),
        [11., 13., 15.],
    );
    sdf.validate_non_frame_poses().unwrap();
}

#[path = "support/document_light_projection.rs"]
mod projection;
#[test]
fn projected_fixture_retains_complete_native_getters_and_queries() {
    let sdf = load_str(
        include_str!("fixtures/light_documents_original.sdf"),
        "projection.sdf",
        &NoResources,
    )
    .unwrap();
    let data: serde_json::Value =
        serde_json::from_str(include_str!("document_lights.json")).unwrap();
    let world = &sdf.worlds[0];
    let model = &world.models[0];
    let link = &model.links[0];
    let lights = [
        &world.lights[0],
        &world.lights[1],
        &world.lights[2],
        &link.lights[0],
    ];
    let lines = data["projection"]["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 18);
    for (line, light) in lines[..4].iter().zip(lights) {
        let row: Vec<String> = line
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        assert_eq!(row.len(), 35);
        assert_eq!(row[0], "LIGHT");
        projection::compare(&projection::projection(light), &row[2..]);
    }
    let raw: Vec<String> = lines[4]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().into())
        .collect();
    assert_eq!(raw.len(), 10);
    assert_eq!(raw[0], "MODEL");
    assert_eq!(raw[1], projection::hex(&model.name));
    assert_eq!(raw[2], projection::hex(&model.pose.relative_to));
    projection::pose(model.pose.transform, &raw[3..]);
    for line in &lines[5..] {
        let row: Vec<String> = line
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().into())
            .collect();
        assert_eq!(row[0], "POSE");
        let name = row[1].as_str();
        let result = match name {
            "model_frame1" => sdf.pose_of(&model.scoped_name, &format!("{}::frame1", world.name)),
            "model_world" | "model_default" => sdf.pose_of(&model.scoped_name, &world.name),
            _ => {
                let (light, scope, host, target) = match name {
                    "point_world" | "point_default" => (
                        lights[0],
                        world.name.as_str(),
                        world.name.as_str(),
                        world.name.clone(),
                    ),
                    "point_unknown" => (
                        lights[0],
                        world.name.as_str(),
                        world.name.as_str(),
                        format!("{}::missing", world.name),
                    ),
                    "directional_frame1" => (
                        lights[2],
                        world.name.as_str(),
                        world.name.as_str(),
                        format!("{}::frame1", world.name),
                    ),
                    "directional_world" | "directional_default" => (
                        lights[2],
                        world.name.as_str(),
                        world.name.as_str(),
                        world.name.clone(),
                    ),
                    "link_frame2" => (
                        lights[3],
                        model.scoped_name.as_str(),
                        link.scoped_name.as_str(),
                        format!("{}::frame2", model.scoped_name),
                    ),
                    "link_model" => (
                        lights[3],
                        model.scoped_name.as_str(),
                        link.scoped_name.as_str(),
                        model.scoped_name.clone(),
                    ),
                    "link_default" => (
                        lights[3],
                        model.scoped_name.as_str(),
                        link.scoped_name.as_str(),
                        link.scoped_name.clone(),
                    ),
                    "link_unknown" => (
                        lights[3],
                        model.scoped_name.as_str(),
                        link.scoped_name.as_str(),
                        format!("{}::missing", model.scoped_name),
                    ),
                    _ => panic!("unknown native query {name}"),
                };
                sdf.resolve_pose(&light.pose, scope, host, &target)
            }
        };
        if row[2] == "REJECT" {
            assert_eq!(row.len(), 3);
            assert_eq!(result.unwrap_err().code, ErrorCode::UnknownReference);
        } else {
            assert_eq!(row.len(), 10);
            projection::pose(result.unwrap(), &row[3..]);
        }
    }
}
