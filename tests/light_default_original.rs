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
// Fixed value observations from DOMLight.DefaultConstruction.
use glam::DQuat;
use sdformat::{Light, LightType, Transform};

#[test]
fn partial_dom_light_default_construction_values() {
    let mut light = Light::default();
    assert_eq!(light.light_type, LightType::Point);
    assert!(light.name.is_empty());

    light.name = "test_light".into();
    assert_eq!(light.name, "test_light");

    assert_eq!(light.pose.transform, Transform::default());
    assert!(light.pose.relative_to.is_empty());
    let semantic_pose = light.pose.clone();
    assert_eq!(semantic_pose.transform, light.pose.transform);
    assert!(semantic_pose.relative_to.is_empty());

    light.pose.transform.translation = [1., 2., 3.];
    light.pose.transform.rotation = DQuat::from_rotation_z(std::f64::consts::PI).to_array();
    assert_eq!(light.pose.transform.translation, [1., 2., 3.]);
    let rotation = light.pose.transform.rotation;
    assert_eq!(rotation[0], 0.);
    assert_eq!(rotation[1], 0.);
    assert!((rotation[2] - 1.).abs() < 1e-15);
    assert!(rotation[3].abs() < 1e-15);

    light.pose.relative_to = "world".into();
    assert_eq!(light.pose.relative_to, "world");
    let semantic_pose = light.pose.clone();
    assert_eq!(semantic_pose.transform, light.pose.transform);
    assert_eq!(semantic_pose.relative_to, "world");

    assert!(light.light_on);
    light.light_on = false;
    assert!(!light.light_on);

    assert!(light.visualize);
    light.visualize = false;
    assert!(!light.visualize);

    assert!(!light.cast_shadows);
    light.cast_shadows = true;
    assert!(light.cast_shadows);

    assert_eq!(light.diffuse, [0., 0., 0., 1.]);
    light.diffuse = [0.1, 0.2, 0.3, 1.];
    assert_eq!(light.diffuse, [0.1, 0.2, 0.3, 1.]);

    assert_eq!(light.specular, [0., 0., 0., 1.]);
    light.specular = [0.4, 0.6, 0.7, 1.];
    assert_eq!(light.specular, [0.4, 0.6, 0.7, 1.]);

    assert_eq!(light.attenuation_range, 10.);
    light.attenuation_range = 1.2;
    assert_eq!(light.attenuation_range, 1.2);

    assert_eq!(light.linear_attenuation_factor, 1.);
    light.set_linear_attenuation_factor(0.2);
    assert_eq!(light.linear_attenuation_factor, 0.2);

    assert_eq!(light.constant_attenuation_factor, 1.);
    light.set_constant_attenuation_factor(0.4);
    assert_eq!(light.constant_attenuation_factor, 0.4);

    assert_eq!(light.quadratic_attenuation_factor, 0.);
    light.set_quadratic_attenuation_factor(1.1);
    assert_eq!(light.quadratic_attenuation_factor, 1.1);

    assert_eq!(light.direction, [0., 0., -1.]);
    light.direction = [0.4, 0.2, 0.];
    assert_eq!(light.direction, [0.4, 0.2, 0.]);

    assert_eq!(light.spot_inner_angle, 0.);
    light.set_spot_inner_angle(1.4);
    assert_eq!(light.spot_inner_angle, 1.4);

    assert_eq!(light.spot_outer_angle, 0.);
    light.set_spot_outer_angle(0.2);
    assert_eq!(light.spot_outer_angle, 0.2);

    assert_eq!(light.spot_falloff, 0.);
    light.set_spot_falloff(4.3);
    assert_eq!(light.spot_falloff, 4.3);

    assert_eq!(light.intensity, 1.);
    light.intensity = 0.3;
    assert_eq!(light.intensity, 0.3);
}
