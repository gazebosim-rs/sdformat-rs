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
// Fixed values from DOMLight.SpotLightNegativeValues and AttenuationClamp.
use sdformat::Light;

#[test]
fn original_dom_light_spot_light_negative_values() {
    let mut light = Light::default();
    light.set_spot_falloff(-1.0);
    assert_eq!(light.spot_falloff, 0.0);

    light.set_spot_inner_angle(-1.0);
    assert_eq!(light.spot_inner_angle, 0.0);

    light.set_spot_outer_angle(-2.0);
    assert_eq!(light.spot_outer_angle, 0.0);
}

#[test]
fn original_dom_light_attenuation_clamp() {
    let mut light = Light::default();

    light.set_linear_attenuation_factor(-1.0);
    assert_eq!(light.linear_attenuation_factor, 0.0);
    light.set_linear_attenuation_factor(20.0);
    assert_eq!(light.linear_attenuation_factor, 1.0);

    light.set_constant_attenuation_factor(-1.0);
    assert_eq!(light.constant_attenuation_factor, 0.0);
    light.set_constant_attenuation_factor(20.0);
    assert_eq!(light.constant_attenuation_factor, 1.0);

    light.set_quadratic_attenuation_factor(-1.0);
    assert_eq!(light.quadratic_attenuation_factor, 0.0);
}

#[test]
fn supplemental_light_defaults_and_in_range_values() {
    let mut light = Light::default();
    assert_eq!(light.linear_attenuation_factor, 1.0);
    assert_eq!(light.constant_attenuation_factor, 1.0);
    assert_eq!(light.quadratic_attenuation_factor, 0.0);
    assert_eq!(light.spot_falloff, 0.0);
    assert_eq!(light.spot_inner_angle, 0.0);
    assert_eq!(light.spot_outer_angle, 0.0);

    light.set_linear_attenuation_factor(0.2);
    light.set_constant_attenuation_factor(0.4);
    light.set_quadratic_attenuation_factor(1.1);
    light.set_spot_falloff(4.3);
    light.set_spot_inner_angle(1.4);
    light.set_spot_outer_angle(0.2);
    assert_eq!(light.linear_attenuation_factor, 0.2);
    assert_eq!(light.constant_attenuation_factor, 0.4);
    assert_eq!(light.quadratic_attenuation_factor, 1.1);
    assert_eq!(light.spot_falloff, 4.3);
    assert_eq!(light.spot_inner_angle, 1.4);
    assert_eq!(light.spot_outer_angle, 0.2);
}
