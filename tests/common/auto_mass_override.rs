// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;

pub fn cases() -> Vec<Value> {
    let value: Value =
        serde_json::from_str(include_str!("../auto_mass_override_cases.json")).unwrap();
    value["cases"].as_array().unwrap().clone()
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

pub fn assert_mass_override(xml: &str, uri: &str, name: &str) {
    let case = cases().into_iter().find(|row| row["name"] == name).unwrap();
    assert_eq!(case["xml"].as_str().unwrap(), xml);
    assert_eq!(case["uri"].as_str().unwrap(), uri);
    assert!(case.get("rust_error").is_none());
    let mass = if case["mass_text"].as_str().unwrap().trim().is_empty() {
        1.
    } else {
        case["mass_text"].as_str().unwrap().parse::<f64>().unwrap()
    };
    let sdf = load_str(xml, uri, &NoResources).unwrap();
    let manual_xml = xml.replace("auto='true'", "auto='false'");
    let manual = load_str(&manual_xml, uri, &NoResources).unwrap();
    let base_xml = case["base_xml"].as_str().unwrap();
    let base = load_str(base_xml, uri, &NoResources).unwrap();
    assert_eq!(sdf.version, "1.12");
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!(model.name, manual.models[0].name);
    assert_eq!(model.scoped_name, model.name);
    assert_eq!(model.pose, Pose::default());
    assert!(!(model.is_static || model.self_collide || model.enable_wind));
    assert!(model.allow_auto_disable);
    assert!(model.raw_namespace.is_none());
    assert!(model.canonical_link.is_empty() && model.placement_frame.is_empty());
    assert!(model.include.is_none() && model.merged_includes.is_empty());
    assert!(model.models.is_empty() && model.frames.is_empty() && model.joints.is_empty());
    assert!(model.plugins.is_empty() && model.extensions.is_empty());
    assert_eq!(model.element(), manual.models[0].element());
    assert_eq!(model.element().unwrap().children().len(), 1);
    assert_eq!(model.element().unwrap().children()[0].tag(), "link");
    assert_eq!(model.links.len(), 1);
    let link = &model.links[0];
    assert_eq!(link.name, manual.models[0].links[0].name);
    assert_eq!(link.scoped_name, format!("{}::{}", model.name, link.name));
    assert_eq!(link.pose, Pose::default());
    assert_eq!(link.properties, LinkProperties::default());
    assert!(link.velocity_decay.is_none());
    assert!(link.visuals.is_empty() && link.sensors.is_empty() && link.lights.is_empty());
    assert!(
        link.particle_emitters.is_empty() && link.projectors.is_empty() && link.plugins.is_empty()
    );
    assert_eq!(link.collisions.len(), 1);
    let collision = &link.collisions[0];
    let manual_model = &manual.models[0];
    let manual_link = &manual_model.links[0];
    let manual_collision = &manual_link.collisions[0];
    assert_eq!(collision.name, manual_collision.name);
    assert_eq!(collision.pose, Pose::default());
    assert_eq!(collision.pose, manual_collision.pose);
    assert_eq!(collision.geometry, manual_collision.geometry);
    assert!(matches!(
        collision.geometry,
        Geometry::Box { .. } | Geometry::Sphere { .. }
    ));
    assert_eq!(collision.density, manual_collision.density);
    assert_eq!(collision.density(), collision.density.unwrap_or(1000.));
    assert_eq!(collision.surface, Surface::default());
    assert_eq!(collision.surface, manual_collision.surface);
    for (source, text, tag) in [
        (&model.source, xml, "model"),
        (&link.source, xml, "link"),
        (&collision.source, xml, "collision"),
        (&manual_model.source, manual_xml.as_str(), "model"),
        (&manual_link.source, manual_xml.as_str(), "link"),
        (&manual_collision.source, manual_xml.as_str(), "collision"),
    ] {
        assert_eq!(source.uri, uri);
        assert_eq!(source.line, 1);
        assert_eq!(
            source.column,
            u32::try_from(text.find(&format!("<{tag}")).unwrap() + 1).unwrap()
        );
    }
    let default = Inertial::default();
    assert_eq!(manual_link.inertial.mass, mass);
    assert_eq!(manual_link.inertial.inertia, default.inertia);
    assert_eq!(manual_link.inertial.pose, default.pose);
    assert_eq!(
        manual_link.inertial.fluid_added_mass,
        default.fluid_added_mass
    );
    assert_eq!(
        link.inertial.mass, mass,
        "Requested mass is an owned value, not a reciprocal roundtrip"
    );
    assert_eq!(link.inertial.pose, Pose::default());
    assert!(link.inertial.fluid_added_mass.is_none());
    let native: Value =
        serde_json::from_str(include_str!("../auto_mass_override_native.json")).unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        24,
        "Complete actual native observations required"
    );
    let row = rows.iter().find(|row| row["name"] == name).unwrap();
    for key in ["load_errors", "collision_errors", "resolve_errors"] {
        assert!(row[key].as_array().unwrap().is_empty());
    }
    assert_eq!(row["mass_specified"], 1);
    assert_eq!(row["direct_warn_default"], 0);
    assert_eq!(row["stderr"].as_str().unwrap(), "");
    assert_eq!(row["root_status"], serde_json::json!([[1, 1], [1, 1]]));
    assert_eq!(row["collision_status"], serde_json::json!([1, 1]));
    assert_eq!(row["root_values"][0], row["root_values"][1]);
    let expected = row["root_values"][1].as_array().unwrap();
    assert_eq!(expected.len(), 18);
    assert_eq!(&expected[..2], &[Value::from(1), Value::from(1)]);
    assert_eq!(&expected[16..], &[Value::from(0), Value::from(0)]);
    assert_eq!(mass.to_bits(), expected[2].as_u64().unwrap());
    for (actual, expected) in std::iter::once(link.inertial.mass)
        .chain(link.inertial.inertia)
        .chain(link.inertial.pose.transform.translation)
        .chain(link.inertial.pose.transform.rotation)
        .zip(&expected[2..16])
    {
        four_ulps(actual, expected.as_u64().unwrap());
    }
    let direct = row["collision_values"].as_array().unwrap();
    assert_eq!(direct.len(), 14);
    let old = &base.models[0].links[0].inertial;
    for (actual, expected) in std::iter::once(old.mass)
        .chain(old.inertia)
        .chain(old.pose.transform.translation)
        .chain(old.pose.transform.rotation)
        .zip(direct)
    {
        four_ulps(actual, expected.as_u64().unwrap());
    }
    let matrix = row["collision_moi"].as_array().unwrap();
    assert_eq!(matrix.len(), 9);
    for (actual, index) in old.inertia.into_iter().zip([0, 4, 8, 1, 2, 5]) {
        four_ulps(actual, matrix[index].as_u64().unwrap());
    }
    assert_eq!(
        sdf.pose_of(&link.scoped_name, "::").unwrap(),
        Transform::default()
    );
    assert_eq!(
        sdf.resolve_pose(
            &link.inertial.pose,
            &model.scoped_name,
            &link.scoped_name,
            "::"
        )
        .unwrap(),
        Transform::default()
    );
    sdf.validate_non_frame_poses().unwrap();
    let snapshot = format!("{model:?}");
    let mut copy = model.clone();
    assert_eq!(copy.element(), model.element());
    copy.links[0].inertial.mass = 42.;
    copy.links[0].collisions[0].density = Some(7.);
    copy.links[0].collisions[0].source.uri = "edited.sdf".into();
    assert_eq!(format!("{model:?}"), snapshot);
}
