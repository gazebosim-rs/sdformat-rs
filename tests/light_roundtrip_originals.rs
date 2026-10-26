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
// Fixed value observations from DOMLight.ToElement and ToElementErrorOutput.
use glam::DQuat;
use sdformat::{ErrorCode, Light, LightType, load_light_str};

fn source_light() -> Light {
    let mut light = Light {
        name: "test_light_assignment".into(),
        light_type: LightType::Spot,
        ..Light::default()
    };
    light.pose.transform.translation = [3., 2., 1.];
    light.pose.transform.rotation = DQuat::from_rotation_y(std::f64::consts::PI).to_array();
    light.pose.relative_to = "ground_plane".into();
    light.cast_shadows = true;
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

fn assert_loaded(light: &Light) {
    assert_eq!(light.name, "test_light_assignment");
    assert_eq!(light.light_type, LightType::Spot);
    assert_eq!(light.pose.transform.translation, [3., 2., 1.]);
    let expected = DQuat::from_rotation_y(std::f64::consts::PI);
    let actual = DQuat::from_array(light.pose.transform.rotation);
    assert!((actual.dot(expected).abs() - 1.).abs() < 1e-12);
    assert_eq!(light.pose.relative_to, "ground_plane");
    assert!(light.cast_shadows);
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
fn partial_dom_light_to_element_values() {
    let light = source_light();
    let xml = light.to_xml().expect("bounded Light output");
    assert!(xml.starts_with("<light "));
    let mut light2 = load_light_str(&xml, "light-roundtrip").expect("bounded Light load");
    assert_loaded(&light2);

    light2.cast_shadows = false;
    let changed_xml = light2.to_xml().expect("updated Light output");
    let light3 = load_light_str(&changed_xml, "light-roundtrip").expect("updated Light load");
    assert!(!light3.cast_shadows);
}

#[test]
fn partial_dom_light_to_element_error_output_values() {
    let light = source_light();
    let xml = light.to_xml().expect("native Errors.empty value mapping");
    let light2 = load_light_str(&xml, "light-roundtrip").expect("native load errors empty");
    assert_loaded(&light2);
}

#[test]
fn supplemental_light_xml_boundaries() {
    let mut light = Light {
        name: "n & \"quoted\"".into(),
        ..Light::default()
    };
    let xml = light.to_xml().unwrap();
    assert!(xml.contains("&amp;"));
    assert!(xml.contains("&quot;"));
    assert_eq!(load_light_str(&xml, "test").unwrap().name, light.name);

    light.light_type = LightType::Directional;
    assert_eq!(
        load_light_str(&light.to_xml().unwrap(), "test")
            .unwrap()
            .light_type,
        LightType::Directional
    );
    light.name.clear();
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    light.name = "valid".into();
    light.intensity = f64::NAN;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    light.intensity = 1.;
    light.diffuse[0] = f32::INFINITY;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    light.diffuse[0] = 0.;
    light.pose.transform.rotation = [0., 0., 0., 0.];
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);

    assert_eq!(
        load_light_str("<world/>", "test").unwrap_err().code,
        ErrorCode::Xml
    );
    assert!(load_light_str("<light", "test").is_err());
    assert_eq!(
        load_light_str("<light name='n' type='bad'/>", "test")
            .unwrap_err()
            .code,
        ErrorCode::InvalidValue
    );
    assert_eq!(
        load_light_str("<light name='n' type='spot'/>", "test")
            .unwrap_err()
            .code,
        ErrorCode::Missing
    );
    assert_eq!(
        load_light_str("<light name='n'><attenuation/></light>", "test")
            .unwrap_err()
            .code,
        ErrorCode::Missing
    );
}

#[test]
fn supplemental_reject_nonroundtrippable_public_values() {
    let base = Light {
        name: "valid".into(),
        ..Light::default()
    };
    for name in [" world", "world", "__bad__", "a::b", "a\nb", "a\u{1}b"] {
        let mut light = base.clone();
        light.name = name.into();
        assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    let mut light = base.clone();
    light.diffuse[0] = 1.5;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let mut light = base.clone();
    light.specular[1] = -0.1;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let mut light = base.clone();
    light.pose.relative_to = "a\tb".into();
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let mut light = base.clone();
    light.attenuation_range = -1.;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    for value in [-0.1, 1.1] {
        let mut light = base.clone();
        light.linear_attenuation_factor = value;
        assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
        let mut light = base.clone();
        light.constant_attenuation_factor = value;
        assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    }
    let mut light = base.clone();
    light.quadratic_attenuation_factor = -1.;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let mut light = base.clone();
    light.spot_inner_angle = -1.;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let mut light = base.clone();
    light.spot_outer_angle = -1.;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
    let mut light = base;
    light.spot_falloff = -1.;
    assert_eq!(light.to_xml().unwrap_err().code, ErrorCode::InvalidValue);
}
