// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
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
use sdformat::*;

#[test]
fn native_fluid_presence_defaults_and_entry_bits() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("fluid.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 40);
    for case in cases.as_array().unwrap() {
        let result = load_str(case["xml"].as_str().unwrap(), "fluid.sdf", &NoResources);
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{case}"
        );
        if let Ok(sdf) = result {
            let matrix = &sdf.models[0].links[0].inertial.fluid_added_mass;
            let bits = matrix.as_ref().map(|matrix| {
                matrix
                    .iter()
                    .flatten()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>()
            });
            assert_eq!(serde_json::json!(bits), case["matrix_bits"], "{case}");
        }
    }
}

#[test]
fn inertial_complete_original_assertions() {
    // Successful typed loading also enforces the original MassMatrix::IsValid
    // observation: physical inertia is validated before an Inertial is returned.
    let checked = load_str(
        include_str!("fixtures/inertial_complete.sdf"),
        "inertial_complete.sdf",
        &NoResources,
    );
    assert!(checked.is_ok());
    let sdf = checked.unwrap();
    let model = sdf.models.first().unwrap();
    let link = model.links.first().unwrap();
    let inertial = &link.inertial;
    assert_eq!(inertial.mass, 17.982);
    assert_eq!(
        inertial.inertia,
        [0.125569, 0.0972062, 0.117937, 0.0008, -0.000499757, -0.0005]
    );
    assert_eq!(inertial.pose.transform.translation, [0.01, 0., 0.02]);
    let added = inertial.fluid_added_mass.as_ref().unwrap();
    let expected = [
        [11., 12., 13., 14., 15., 16.],
        [12., 22., 23., 24., 25., 26.],
        [13., 23., 33., 34., 35., 36.],
        [14., 24., 34., 44., 45., 46.],
        [15., 25., 35., 45., 55., 56.],
        [16., 26., 36., 46., 56., 66.],
    ];
    // Exact equality strengthens all 36 original EXPECT_DOUBLE_EQ observations.
    assert_eq!(added.as_ref(), &expected);
}

#[test]
fn fluid_schema_boundaries_remain_explicit() {
    for (content, code) in [
        ("<fluid_added_mass attr='x'/>", ErrorCode::Unsupported),
        (
            "<fluid_added_mass><zx>1</zx></fluid_added_mass>",
            ErrorCode::Unsupported,
        ),
        (
            "<fluid_added_mass><xy attr='x'>1</xy></fluid_added_mass>",
            ErrorCode::Unsupported,
        ),
        (
            "<fluid_added_mass><xy>1</xy><xy>2</xy></fluid_added_mass>",
            ErrorCode::InvalidValue,
        ),
        (
            "<fluid_added_mass/><fluid_added_mass/>",
            ErrorCode::InvalidValue,
        ),
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><inertial>{content}</inertial></link></model></sdf>"
        );
        assert_eq!(
            load_str(&xml, "invalid-fluid", &NoResources)
                .unwrap_err()
                .code,
            code
        );
    }
}
