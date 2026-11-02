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
#[path = "common/auto_mass_override.rs"]
mod mass_override;
use sdformat::*;
#[path = "common/auto_default_density.rs"]
mod default_density;
#[path = "common/auto_sphere.rs"]
mod sphere;

fn values(inertial: &Inertial) -> Vec<u64> {
    std::iter::once(inertial.mass)
        .chain(inertial.inertia)
        .chain(inertial.pose.transform.translation)
        .chain(inertial.pose.transform.rotation)
        .map(f64::to_bits)
        .collect()
}

#[path = "common/auto_identity_pose.rs"]
mod identity_pose;

#[path = "common/auto_translation_pose.rs"]
mod translation;

#[test]
fn original_correct_box_collision_calculates_owned_inertial() {
    let xml = include_str!("fixtures/auto_box_inertial_original.sdf");
    let sdf = load_str(xml, "auto-box-original.sdf", &NoResources).unwrap();
    assert_eq!(sdf.version, "1.11");
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!(model.name, "shapes");
    assert_eq!(model.scoped_name, "shapes");
    assert_eq!(model.pose, Pose::default());
    assert!(!(model.is_static || model.self_collide || model.enable_wind));
    assert!(model.allow_auto_disable);
    assert!(model.raw_namespace.is_none());
    assert!(model.canonical_link.is_empty());
    assert!(model.placement_frame.is_empty());
    assert!(model.include.is_none());
    assert!(model.merged_includes.is_empty());
    assert!(model.models.is_empty());
    assert!(model.frames.is_empty());
    assert!(model.joints.is_empty());
    assert!(model.plugins.is_empty());
    assert!(model.extensions.is_empty());
    let children = model.element().unwrap().children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].tag(), "link");
    assert_eq!(children[0].name_attribute(), Some("link"));
    assert_eq!(model.links.len(), 1);
    let link = &model.links[0];
    assert_eq!(link.name, "link");
    assert_eq!(link.scoped_name, "shapes::link");
    assert_eq!(link.pose, Pose::default());
    assert_eq!(link.properties, LinkProperties::default());
    assert!(link.velocity_decay.is_none());
    assert!(link.visuals.is_empty());
    assert!(link.sensors.is_empty());
    assert!(link.lights.is_empty());
    assert!(link.particle_emitters.is_empty());
    assert!(link.projectors.is_empty());
    assert!(link.plugins.is_empty());
    assert_eq!(link.collisions.len(), 1);
    let collision = &link.collisions[0];
    assert_eq!(collision.name, "box_col");
    assert_eq!(collision.density, Some(1240.));
    assert_eq!(collision.density(), 1240.);
    assert_eq!(collision.pose, Pose::default());
    assert!(matches!(
        collision.geometry,
        Geometry::Box { size: [2., 2., 2.] }
    ));
    assert_eq!(collision.surface, Surface::default());
    for (source, tag) in [
        (&model.source, "model"),
        (&link.source, "link"),
        (&collision.source, "collision"),
    ] {
        assert_eq!(source.uri, "auto-box-original.sdf");
        assert_eq!(source.line, 1);
        assert_eq!(
            source.column,
            u32::try_from(xml.find(&format!("<{tag}")).unwrap() + 1).unwrap()
        );
    }
    // This original box is exact even through Parry's reciprocal representation.
    assert_eq!(link.inertial.mass.to_bits(), 4666679192002232320);
    assert_eq!(
        link.inertial.inertia.map(f64::to_bits),
        [
            4663993451599451477,
            4663993451599451477,
            4663993451599451477,
            0,
            0,
            0
        ]
    );
    assert_eq!(link.inertial.pose, Pose::default());
    assert!(link.inertial.fluid_added_mass.is_none());
    assert_eq!(
        sdf.pose_of("shapes::link", "::").unwrap(),
        Transform::default()
    );
    sdf.validate_non_frame_poses().unwrap();
}

#[test]
fn native_auto_box_corpus_retains_numeric_domains_and_source() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("auto_box_inertial.json")).unwrap();
    let mut positives = 0;
    for row in corpus["cases"].as_array().unwrap() {
        if row["name"] == "pose_translated" {
            assert_eq!(row["rust_error"], "Unsupported");
            translation::assert_unit_translation(
                row["xml"].as_str().unwrap(),
                "auto-box.sdf",
                false,
            );
            continue;
        }
        if row["name"] == "pose_identity" {
            identity_pose::assert_identity_pose(row, "auto-box.sdf", false);
            continue;
        }
        if row["name"] == "sphere" {
            assert_eq!(row["rust_error"], "Unsupported");
            sphere::assert_complete_sphere(
                row["xml"].as_str().unwrap(),
                "auto-box.sdf",
                "1.12",
                "box_col",
                (1240., 1., 4662432987886324156, 4656787193621907402),
            );
            continue;
        }
        if matches!(
            row["name"].as_str(),
            Some("default_density" | "empty_density")
        ) {
            assert_eq!(row["rust_error"], "Unsupported");
            let lines: Vec<_> = row["stdout"]
                .as_str()
                .unwrap()
                .lines()
                .filter_map(|line| line.strip_prefix("VALUES "))
                .collect();
            assert_eq!(lines.len(), 2);
            assert_eq!(lines[0], lines[1]);
            let native: Vec<u64> = lines[0]
                .split_whitespace()
                .map(|v| v.parse().unwrap())
                .collect();
            default_density::assert_complete_default_density(
                row["xml"].as_str().unwrap(),
                "auto-box.sdf",
                "1.12",
                "box",
                (row["name"] == "empty_density").then_some(1000.),
                &native,
            );
            continue;
        }
        if row["name"] == "mass_override" {
            assert_eq!(row["rust_error"], "Unsupported");
            mass_override::assert_mass_override(
                row["xml"].as_str().unwrap(),
                "auto-box.sdf",
                "box_old_corpus",
            );
            continue;
        }
        let result = load_str(row["xml"].as_str().unwrap(), "auto-box.sdf", &NoResources);
        if let Some(expected) = row.get("rust_error") {
            let code = match expected.as_str().unwrap() {
                "Unsupported" => ErrorCode::Unsupported,
                "InvalidValue" => ErrorCode::InvalidValue,
                _ => panic!("unknown corpus error"),
            };
            let error = result
                .err()
                .unwrap_or_else(|| panic!("{} was accepted", row["name"]));
            assert_eq!(error.code, code, "{}", row["name"]);
            assert_eq!(error.source.uri, "auto-box.sdf");
            assert!(error.source.line > 0 && error.source.column > 0);
            continue;
        }
        let sdf = result.unwrap_or_else(|e| panic!("{}: {e}", row["name"]));
        let lines: Vec<_> = row["stdout"].as_str().unwrap().lines().collect();
        assert_eq!(lines[0], "LOAD");
        assert_eq!(lines[2], "RESOLVE");
        assert_eq!(lines[1], lines[3]);
        let native: Vec<u64> = lines[1]
            .strip_prefix("VALUES ")
            .unwrap()
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        assert_eq!(native.len(), 18);
        assert_eq!(&native[..2], [1, 1]);
        assert_eq!(&native[16..], [0, 0]);
        if matches!(
            row["name"].as_str().unwrap(),
            "zero_x" | "negative_y" | "zero_z" | "default_box_size"
        ) {
            assert!(matches!(
                sdf.models[0].links[0].collisions[0].geometry,
                Geometry::Box { size: [1., 1., 1.] }
            ));
        }
        let inertial = &sdf.models[0].links[0].inertial;
        let actual = values(inertial);
        // Native EXPECT_DOUBLE_EQ permits four ULPs; zero/pose bits stay exact.
        for (actual, expected) in actual[..4].iter().zip(&native[2..6]) {
            assert!(f64::from_bits(*actual) > 0. && f64::from_bits(*expected) > 0.);
            assert!(actual.abs_diff(*expected) <= 4, "{}", row["name"]);
        }
        assert_eq!(&actual[4..], &native[6..16]);
        assert!(inertial.pose.relative_to.is_empty());
        assert!(inertial.fluid_added_mass.is_none());
        let mut cloned = sdf.models[0].clone();
        assert_eq!(values(&cloned.links[0].inertial), actual);
        cloned.links[0].inertial.mass += 1.;
        assert_eq!(values(inertial), actual);
        sdf.validate_non_frame_poses().unwrap();
        positives += 1;
    }
    assert_eq!(positives, 25);
}

#[test]
fn automatic_box_loading_preserves_include_provenance_and_frames() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, _: &str, _: &str) -> Result<Resource, String> {
            Ok(Resource {
                canonical_uri: "boxes/box.sdf".into(),
                xml: include_str!("fixtures/auto_box_inertial_original.sdf").into(),
            })
        }
    }
    let sdf = load_str("<sdf version='1.12'><world name='w'><include><uri>box</uri><name>imported</name><pose>1 2 3 0 0 0</pose></include></world></sdf>", "world.sdf", &Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    assert_eq!(model.name, "imported");
    assert_eq!(model.source.uri, "boxes/box.sdf");
    assert_eq!(model.links[0].source.uri, "boxes/box.sdf");
    assert_eq!(model.links[0].collisions[0].source.uri, "boxes/box.sdf");
    assert_eq!(model.links[0].inertial.pose, Pose::default());
    assert_eq!(model.links[0].inertial.mass, 9920.);
    assert_eq!(
        sdf.pose_of("w::imported::link", "::").unwrap().translation,
        [1., 2., 3.]
    );
    sdf.validate_non_frame_poses().unwrap();
}
