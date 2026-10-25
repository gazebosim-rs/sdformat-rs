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
// Only the original Load value observations below are ported; Element/error paths remain pending.
use sdformat::*;

#[test]
fn native_basic_primitive_fallbacks_and_normalization_bits() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("basic_primitives.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 37);
    for case in cases.as_array().unwrap() {
        let result = load_str(case["xml"].as_str().unwrap(), "basic.sdf", &NoResources);
        assert_eq!(
            result.is_ok(),
            case["accepted"].as_bool().unwrap(),
            "{case}"
        );
        if let Ok(sdf) = result {
            let geometry = &sdf.models[0].links[0].collisions[0].geometry;
            let values = match geometry {
                Geometry::Box { size } => size.to_vec(),
                Geometry::Sphere { radius } => vec![*radius],
                Geometry::Cylinder { radius, length } => vec![*radius, *length],
                Geometry::Plane { normal, size } => normal.iter().chain(size).copied().collect(),
                _ => panic!("basic primitive expected"),
            };
            let bits: Vec<_> = values.into_iter().map(f64::to_bits).collect();
            assert_eq!(serde_json::json!(bits), case["bits"], "{case}");
        }
    }
}

#[test]
fn formerly_rejected_native_defaults_remain_explicit_positive_regressions() {
    for (shape, sphere) in [
        ("<sphere><radius>-1</radius></sphere>", true),
        ("<plane><normal>0 0 0</normal></plane>", false),
    ] {
        let xml = format!(
            "<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><geometry>{shape}</geometry></collision></link></model></sdf>"
        );
        let sdf = load_str(&xml, "defaults.sdf", &NoResources).unwrap();
        if sphere {
            assert!(matches!(
                sdf.models[0].links[0].collisions[0].geometry,
                Geometry::Sphere { radius: 1. }
            ));
        } else {
            assert!(matches!(
                sdf.models[0].links[0].collisions[0].geometry,
                Geometry::Plane {
                    normal: [0., 0., 0.],
                    size: [1., 1.]
                }
            ));
        }
    }
}

fn shape(xml: &str) -> Geometry {
    let sdf = load_str(&format!("<sdf version='1.12'><model name='m'><link name='l'><collision name='c'><geometry>{xml}</geometry></collision></link></model></sdf>"), "original-shape", &NoResources).unwrap();
    sdf.models
        .into_iter()
        .next()
        .unwrap()
        .links
        .into_iter()
        .next()
        .unwrap()
        .collisions
        .into_iter()
        .next()
        .unwrap()
        .geometry
}

#[test]
fn box_load_original_negative_value_subset() {
    let Geometry::Box { size } = shape("<box><size>-1 -1 -1</size></box>") else {
        panic!("box")
    };
    for value in size {
        assert!((value - 1.).abs() <= 1e-3);
    }
    assert_eq!(size, [1.; 3]);
}

#[test]
fn sphere_load_original_value_subset() {
    assert!(matches!(
        shape("<sphere><radius>2.5</radius></sphere>"),
        Geometry::Sphere { radius: 2.5 }
    ));
    assert!(matches!(
        shape("<sphere><radius>-1</radius></sphere>"),
        Geometry::Sphere { radius: 1. }
    ));
}

#[test]
fn cylinder_load_original_negative_value_subset() {
    let Geometry::Cylinder { radius, .. } =
        shape("<cylinder><radius>-1</radius><length>3</length></cylinder>")
    else {
        panic!("cylinder")
    };
    assert_eq!(radius, 0.5);
    let Geometry::Cylinder { length, .. } =
        shape("<cylinder><radius>1</radius><length>-1</length></cylinder>")
    else {
        panic!("cylinder")
    };
    assert_eq!(length, 1.);
}

#[test]
fn plane_load_original_negative_value_subset() {
    let Geometry::Plane { size, .. } =
        shape("<plane><normal>1 0 0</normal><size>-1 -1</size></plane>")
    else {
        panic!("plane")
    };
    for value in size {
        assert!((value - 1.).abs() <= 1e-3);
    }
    assert_eq!(size, [1.; 2]);
}
