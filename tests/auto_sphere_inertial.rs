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
 */
// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
#[path = "common/auto_sphere.rs"]
mod common;
#[path = "common/auto_default_density.rs"]
mod default_density;

#[test]
fn original_sphere_inertial_values_stay_exact() {
    // Derived loader fixture: the original uses a mutable Sphere and no XML.
    common::assert_complete_sphere(
        include_str!("fixtures/auto_sphere_inertial.sdf"),
        "sphere-original.sdf",
        "1.11",
        "sphere_col",
        (2170., 0.1, 4621306650028725639, 4585400661777136886),
    );
}

fn interval(value: f64, bounds: &serde_json::Value) {
    let low = f64::from_bits(bounds[0].as_u64().unwrap());
    let high = f64::from_bits(bounds[1].as_u64().unwrap());
    assert!(
        value >= low && value <= high,
        "{value:e} outside [{low:e}, {high:e}]"
    );
}

#[test]
fn sphere_loader_native_domains_and_oracle_roundoff() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("auto_sphere_inertial.json")).unwrap();
    let mut oracle_cases = 0;
    for row in corpus["cases"].as_array().unwrap() {
        let xml = row["xml"].as_str().unwrap();
        let uri = row["uri"].as_str().unwrap_or("sphere.sdf");
        let result = load_str(xml, uri, &NoResources);
        if matches!(
            row["name"].as_str().unwrap(),
            "no_density" | "empty_density"
        ) {
            assert_eq!(row["rust_error"], "Unsupported");
            let lines: Vec<_> = row["stdout"].as_str().unwrap().lines().collect();
            assert_eq!(lines.len(), 4);
            assert_eq!(lines[0], "LOAD");
            assert_eq!(lines[2], "RESOLVE");
            assert_eq!(lines[1], lines[3]);
            let expected: Vec<u64> = lines[1]
                .strip_prefix("VALUES ")
                .unwrap()
                .split_whitespace()
                .map(|value| value.parse().unwrap())
                .collect();
            default_density::assert_complete_default_density(
                row["xml"].as_str().unwrap(),
                "sphere.sdf",
                "1.12",
                "sphere_default",
                (row["name"] == "empty_density").then_some(1000.),
                &expected,
            );
            continue;
        }
        if let Some(error) = row.get("rust_error") {
            let code = match error.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                "Missing" => ErrorCode::Missing,
                _ => panic!("unknown corpus error"),
            };
            let error = result
                .err()
                .unwrap_or_else(|| panic!("{} accepted", row["name"]));
            assert_eq!(error.code, code, "{}", row["name"]);
            assert_eq!(error.source.uri, uri);
            assert!(error.source.line > 0 && error.source.column > 0);
            continue;
        }
        let sdf = result.unwrap_or_else(|e| panic!("{}: {e}", row["name"]));
        let inertial = &sdf.models[0].links[0].inertial;
        let lines: Vec<_> = row["stdout"].as_str().unwrap().lines().collect();
        assert_eq!(lines[0], "LOAD");
        assert_eq!(lines[2], "RESOLVE");
        assert_eq!(lines[1], lines[3]);
        let values: Vec<u64> = lines[1]
            .strip_prefix("VALUES ")
            .unwrap()
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        assert_eq!(values.len(), 18);
        assert_eq!(&values[..2], [1, 1]);
        assert_eq!(&values[16..], [0, 0]);
        assert_eq!(inertial.inertia[0], inertial.inertia[1]);
        assert_eq!(inertial.inertia[0], inertial.inertia[2]);
        if let Some(bounds) = row.get("oracle_intervals") {
            for (i, actual) in [inertial.mass, inertial.inertia[0]].into_iter().enumerate() {
                let native = f64::from_bits(values[2 + i]);
                interval(actual, &bounds["parry"][i]);
                interval(native, &bounds["native"][i]);
                assert!(
                    (actual - native).abs()
                        <= f64::from_bits(row["differential_absolute_bounds"][i].as_u64().unwrap()),
                    "{}",
                    row["name"]
                );
            }
            oracle_cases += 1;
        } else {
            assert_eq!(inertial.mass.to_bits(), values[2], "{}", row["name"]);
            assert_eq!(
                inertial.inertia[..3]
                    .iter()
                    .map(|v| v.to_bits())
                    .collect::<Vec<_>>(),
                values[3..6]
            );
        }
        let tail: Vec<_> = inertial.inertia[3..]
            .iter()
            .copied()
            .chain(inertial.pose.transform.translation)
            .chain(inertial.pose.transform.rotation)
            .map(f64::to_bits)
            .collect();
        assert_eq!(tail, values[6..16]);
        assert_eq!(inertial.pose, Pose::default());
        assert!(inertial.fluid_added_mass.is_none());
        if matches!(
            row["name"].as_str().unwrap(),
            "zero_radius" | "negative_radius" | "default_radius"
        ) {
            assert_eq!(
                sdf.models[0].links[0].collisions[0].geometry,
                Geometry::Sphere { radius: 1. }
            );
        }
        let mut copy = sdf.models[0].clone();
        copy.links[0].inertial.mass = 0.;
        assert_ne!(
            copy.links[0].inertial.mass.to_bits(),
            inertial.mass.to_bits()
        );
        sdf.validate_non_frame_poses().unwrap();
    }
    assert_eq!(oracle_cases, 55);
}

#[test]
fn sphere_includes_retain_resource_identity_and_frame_pose() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
            Ok(Resource {
                canonical_uri: "spheres/model.sdf".into(),
                xml: include_str!("fixtures/auto_sphere_inertial.sdf").into(),
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>sphere</uri><name>imported</name><pose>1 2 3 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    assert_eq!(model.source.uri, "spheres/model.sdf");
    assert_eq!(model.links[0].source.uri, model.source.uri);
    assert_eq!(model.links[0].collisions[0].source.uri, model.source.uri);
    assert_eq!(model.links[0].inertial.mass.to_bits(), 4621306650028725639);
    assert_eq!(model.links[0].inertial.pose, Pose::default());
    assert_eq!(
        sdf.pose_of("w::imported::link", "::").unwrap().translation,
        [1., 2., 3.]
    );
}
