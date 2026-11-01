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

pub fn assert_complete_default_density(
    xml: &str,
    uri: &str,
    version: &str,
    shape: &str,
    density: Option<f64>,
    expected: &[u64],
) {
    let case = match shape {
        "box" => Case {
            names: ["shapes", "link", "box_col"],
            geometry: Geometry::Box { size: [2.; 3] },
            sphere_oracle: None,
        },
        "sphere" => Case {
            names: ["shapes", "link", "box_col"],
            geometry: Geometry::Sphere { radius: 1. },
            sphere_oracle: Some(""),
        },
        "sphere_default" => Case {
            names: ["shapes", "link", "sphere_col"],
            geometry: Geometry::Sphere { radius: 0.1 },
            sphere_oracle: Some("dependent_sphere"),
        },
        "manual_box" => Case {
            names: ["m", "l", "c"],
            geometry: Geometry::Box { size: [1.; 3] },
            sphere_oracle: None,
        },
        other => panic!("unexpected fixture: {other}"),
    };
    assert_complete(xml, uri, version, density, expected, case);
}

struct Case {
    names: [&'static str; 3],
    geometry: Geometry,
    sphere_oracle: Option<&'static str>,
}

fn assert_complete(
    xml: &str,
    uri: &str,
    version: &str,
    density: Option<f64>,
    expected: &[u64],
    case: Case,
) {
    let [model_name, link_name, collision_name] = case.names;
    let sdf = load_str(xml, uri, &NoResources).unwrap();
    assert_eq!(sdf.version, version);
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!(model.name, model_name);
    assert_eq!(model.scoped_name, model_name);
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
    assert_eq!(children[0].name_attribute(), Some(link_name));
    assert_eq!(model.links.len(), 1);
    let link = &model.links[0];
    assert_eq!(link.name, link_name);
    assert_eq!(link.scoped_name, format!("{model_name}::{link_name}"));
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
    assert_eq!(collision.name, collision_name);
    assert_eq!(collision.density, density);
    assert_eq!(collision.density(), 1000.);
    assert_eq!(collision.pose, Pose::default());
    assert_eq!(collision.geometry, case.geometry);
    assert_eq!(collision.surface, Surface::default());
    for (source, tag) in [
        (&model.source, "model"),
        (&link.source, "link"),
        (&collision.source, "collision"),
    ] {
        assert_eq!(source.uri, uri);
        assert_eq!(source.line, 1);
        assert_eq!(
            source.column,
            u32::try_from(xml.find(&format!("<{tag}")).unwrap() + 1).unwrap()
        );
    }
    assert_eq!(expected.len(), 18);
    assert_eq!(&expected[..2], [1, 1]);
    assert_eq!(&expected[16..], [0, 0]);
    let actual = [link.inertial.mass, link.inertial.inertia[0]];
    if let Some(key) = case.sphere_oracle {
        let records: serde_json::Value =
            serde_json::from_str(include_str!("../auto_default_density_oracle.json")).unwrap();
        let oracle = if key.is_empty() {
            &records
        } else {
            &records[key]
        };
        for (i, actual) in actual.into_iter().enumerate() {
            let native = f64::from_bits(expected[2 + i]);
            for (kind, value) in [("parry", actual), ("native", native)] {
                let bounds = &oracle["oracle_intervals"][kind][i];
                assert!(value >= f64::from_bits(bounds[0].as_u64().unwrap()));
                assert!(value <= f64::from_bits(bounds[1].as_u64().unwrap()));
            }
            assert!(
                (actual - native).abs()
                    <= f64::from_bits(oracle["differential_absolute_bounds"][i].as_u64().unwrap())
            );
        }
        assert_eq!(link.inertial.inertia[0], link.inertial.inertia[1]);
        assert_eq!(link.inertial.inertia[0], link.inertial.inertia[2]);
    } else {
        for (actual, expected) in std::iter::once(link.inertial.mass)
            .chain(link.inertial.inertia[..3].iter().copied())
            .zip(&expected[2..6])
        {
            assert!(actual.to_bits().abs_diff(*expected) <= 4);
        }
    }
    let tail: Vec<_> = link.inertial.inertia[3..]
        .iter()
        .copied()
        .chain(link.inertial.pose.transform.translation)
        .chain(link.inertial.pose.transform.rotation)
        .map(f64::to_bits)
        .collect();
    assert_eq!(tail, expected[6..16]);
    assert_eq!(link.inertial.pose, Pose::default());
    assert!(link.inertial.fluid_added_mass.is_none());
    assert_eq!(
        sdf.pose_of(&format!("{model_name}::{link_name}"), "::")
            .unwrap(),
        Transform::default()
    );
    sdf.validate_non_frame_poses().unwrap();
    let mut copy = model.clone();
    assert_eq!(copy.element(), model.element());
    assert_eq!(copy.links[0].collisions[0].density, density);
    copy.links[0].inertial.mass += 1.;
    copy.links[0].collisions[0].density = Some(5.);
    assert_eq!(collision.density, density);
    assert_ne!(copy.links[0].inertial.mass, link.inertial.mass);
}
