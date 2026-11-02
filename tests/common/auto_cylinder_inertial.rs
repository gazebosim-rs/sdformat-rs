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

pub fn assert_retained_input(xml: &str, uri: &str) {
    let sdf = load_str(xml, uri, &NoResources).unwrap();
    let manual = load_str(
        &xml.replace("auto='true'", "auto='false'"),
        uri,
        &NoResources,
    )
    .unwrap();
    assert_eq!(sdf.version, "1.12");
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!(model.name, "m");
    assert_eq!(model.scoped_name, "m");
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
    assert_eq!(children[0].name_attribute(), Some("l"));
    assert_eq!(model.links.len(), 1);
    let link = &model.links[0];
    assert_eq!(link.name, "l");
    assert_eq!(link.scoped_name, "m::l");
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
    assert_eq!(collision.name, "c");
    assert_eq!(collision.density, None);
    assert_eq!(collision.density(), 1000.);
    assert_eq!(collision.pose.transform.translation, [1., 2., 3.]);
    assert_eq!(collision.pose, manual.models[0].links[0].collisions[0].pose);
    assert!(collision.pose.relative_to.is_empty());
    assert_eq!(
        collision.geometry,
        Geometry::Cylinder {
            radius: 1.,
            length: 2.
        }
    );
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
    let native: serde_json::Value =
        serde_json::from_str(include_str!("../auto_cylinder_inertial_native.json")).unwrap();
    let name = "retained_negative";
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../auto_cylinder_inertial_cases.json")).unwrap();
    let case = cases["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .unwrap();
    assert_eq!(case["xml"].as_str().unwrap(), xml);
    assert_eq!(case["uri"].as_str().unwrap(), uri);
    let rows = native["observations"].as_array().unwrap();
    let row = rows
        .iter()
        .find(|row| row["name"] == name)
        .expect("Actual reviewed native Cylinder data is required");
    assert_eq!(row["root_status"], serde_json::json!([[1, 1], [1, 1]]));
    assert_eq!(row["collision_status"], serde_json::json!([1, 1]));
    assert_eq!(row["collision_moi"].as_array().unwrap().len(), 9);
    let expected = row["root_values"][1].as_array().unwrap();
    assert_eq!(expected.len(), 18);
    assert_eq!(expected[0], 1);
    assert_eq!(expected[1], 1);
    assert_eq!(expected[16], 0);
    assert_eq!(expected[17], 0);
    assert!(row["load_errors"].as_array().unwrap().is_empty());
    assert!(row["resolve_errors"].as_array().unwrap().is_empty());
    assert!(row["collision_errors"].as_array().unwrap().is_empty());
    assert_eq!(row["root_values"][0], row["root_values"][1]);
    four_ulps(link.inertial.mass, expected[2].as_u64().unwrap());
    for (actual, native) in link.inertial.inertia.into_iter().zip(&expected[3..9]) {
        four_ulps(actual, native.as_u64().unwrap());
    }
    for (actual, native) in link
        .inertial
        .pose
        .transform
        .translation
        .into_iter()
        .zip(&expected[9..12])
    {
        four_ulps(actual, native.as_u64().unwrap());
    }
    assert_eq!(
        link.inertial.pose.transform.rotation,
        Transform::default().rotation
    );
    assert!(link.inertial.pose.relative_to.is_empty());
    assert!(link.inertial.fluid_added_mass.is_none());
    assert_eq!(sdf.pose_of("m::l", "::").unwrap(), Transform::default());
    assert_eq!(
        sdf.resolve_pose(&link.inertial.pose, "m", "m::l", "::")
            .unwrap()
            .translation,
        link.inertial.pose.transform.translation
    );
    sdf.validate_non_frame_poses().unwrap();
    let center = link.inertial.pose.transform.translation;
    let mut copied = model.clone();
    assert_eq!(copied.element(), model.element());
    copied.links[0].collisions[0].pose.transform.translation = [2., 0., 0.];
    copied.links[0].inertial.pose.transform.translation = [2., 0., 0.];
    assert_eq!(collision.pose.transform.translation, [1., 2., 3.]);
    assert_eq!(link.inertial.pose.transform.translation, center);
}

pub fn four_ulps(actual: f64, expected: u64) {
    let native = f64::from_bits(expected);
    assert!(actual.is_finite() && native.is_finite());
    if actual == native {
        return;
    }
    assert_eq!(actual.is_sign_negative(), native.is_sign_negative());
    assert!(
        actual.to_bits().abs_diff(expected) <= 4,
        "{actual:e} != {native:e}"
    );
}
