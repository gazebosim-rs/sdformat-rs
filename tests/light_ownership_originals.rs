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
// Fixed final values from five DOMLight copy and move originals.
use glam::DQuat;
use sdformat::{Light, LightType};

fn configured_light(name: &str) -> Light {
    let mut light = Light {
        name: name.into(),
        light_type: LightType::Directional,
        ..Light::default()
    };
    light.pose.transform.translation = [3., 2., 1.];
    light.pose.transform.rotation = DQuat::from_rotation_y(std::f64::consts::PI).to_array();
    light.pose.relative_to = "ground_plane".into();
    light.cast_shadows = true;
    light.light_on = false;
    light.visualize = false;
    light.diffuse = [0.4, 0.5, 0.6, 1.];
    light.specular = [0.8, 0.9, 0.1, 1.];
    light.attenuation_range = 3.2;
    light.set_linear_attenuation_factor(0.1);
    light.set_constant_attenuation_factor(0.5);
    light.set_quadratic_attenuation_factor(0.01);
    light.direction = [0.1, 0.2, 1.];
    light.set_spot_inner_angle(1.9);
    light.set_spot_outer_angle(3.3);
    light.set_spot_falloff(0.9);
    light.intensity = 1.7;
    light
}

fn assert_configured(light: &Light, name: &str) {
    assert_eq!(light.name, name);
    assert_eq!(light.light_type, LightType::Directional);
    assert_eq!(light.pose.transform.translation, [3., 2., 1.]);
    let rotation = light.pose.transform.rotation;
    assert_eq!(rotation[0], 0.);
    assert!((rotation[1] - 1.).abs() < 1e-15);
    assert_eq!(rotation[2], 0.);
    assert!(rotation[3].abs() < 1e-15);
    assert_eq!(light.pose.relative_to, "ground_plane");
    assert!(light.cast_shadows);
    assert!(!light.light_on);
    assert!(!light.visualize);
    assert_eq!(light.diffuse, [0.4, 0.5, 0.6, 1.]);
    assert_eq!(light.specular, [0.8, 0.9, 0.1, 1.]);
    assert_eq!(light.attenuation_range, 3.2);
    assert_eq!(light.linear_attenuation_factor, 0.1);
    assert_eq!(light.constant_attenuation_factor, 0.5);
    assert_eq!(light.quadratic_attenuation_factor, 0.01);
    assert_eq!(light.direction, [0.1, 0.2, 1.]);
    assert_eq!(light.spot_inner_angle, 1.9);
    assert_eq!(light.spot_outer_angle, 3.3);
    assert_eq!(light.spot_falloff, 0.9);
    assert_eq!(light.intensity, 1.7);
}

#[test]
fn original_dom_light_copy_constructor() {
    let light = configured_light("test_copy_light");
    let light2 = light.clone();
    assert_configured(&light2, "test_copy_light");
}

#[test]
fn original_dom_light_copy_assignment_operator() {
    let light = configured_light("test_light_assignment");
    let mut light2 = Light::default();
    light2.clone_from(&light);
    assert_configured(&light2, "test_light_assignment");
}

#[test]
fn original_dom_light_move_constructor() {
    let light = configured_light("test_light_assignment");
    let light2 = light;
    assert_configured(&light2, "test_light_assignment");
}

#[test]
fn original_dom_light_move_assignment() {
    let light = configured_light("test_light_assignment");
    let mut light2 = Light::default();
    let former_destination = std::mem::replace(&mut light2, light);
    assert_eq!(former_destination, Light::default());
    assert_configured(&light2, "test_light_assignment");
}

#[test]
fn original_dom_light_copy_assignment_after_move() {
    let light1 = Light {
        name: "light1".into(),
        ..Light::default()
    };
    let light2 = Light {
        name: "light2".into(),
        ..Light::default()
    };
    let tmp = light1;
    let light1 = light2.clone();
    let light2 = tmp.clone();
    assert_eq!(light1.name, "light2");
    assert_eq!(light2.name, "light1");
}
