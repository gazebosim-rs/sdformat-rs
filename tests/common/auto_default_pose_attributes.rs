// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;

pub fn cases() -> Vec<Value> {
    let data: Value =
        serde_json::from_str(include_str!("../auto_default_pose_attributes_cases.json")).unwrap();
    data["cases"].as_array().unwrap().clone()
}

// Same comparison policy as the accepted automatic primitive helpers.
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

pub fn assert_default_attributes(xml: &str, uri: &str, name: &str) {
    let case = cases().into_iter().find(|row| row["name"] == name).unwrap();
    assert_eq!(case["xml"].as_str().unwrap(), xml);
    assert_eq!(case["uri"].as_str().unwrap(), uri);
    let sdf = load_str(xml, uri, &NoResources).unwrap();
    let plain = load_str(case["base_xml"].as_str().unwrap(), uri, &NoResources).unwrap();
    let manual_xml = xml.replace("auto='true'", "auto='false'");
    let manual = load_str(&manual_xml, uri, &NoResources).unwrap();
    assert_eq!(sdf.version, "1.12");
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    assert_eq!(plain.models.len(), 1);
    let model = &sdf.models[0];
    let expected = &plain.models[0];
    // Model owns all Link/Collision fields and its immutable direct-child view.
    // Only accepted default pose attributes differ in these exact source inputs.
    assert_eq!(format!("{model:?}"), format!("{expected:?}"));
    assert_eq!(model.element(), expected.element());
    assert_eq!(model.links.len(), 1);
    let link = &model.links[0];
    assert_eq!(link.collisions.len(), 1);
    let collision = &link.collisions[0];
    let manual_collision = &manual.models[0].links[0].collisions[0];
    assert_eq!(collision.name, manual_collision.name);
    assert_eq!(collision.density, manual_collision.density);
    assert_eq!(collision.pose, manual_collision.pose);
    assert_eq!(collision.geometry, manual_collision.geometry);
    assert_eq!(collision.surface, manual_collision.surface);
    // auto='false' is longer, so its collision starts one source column later.
    assert_eq!(manual_collision.source.uri, uri);
    assert_eq!(manual_collision.source.line, 1);
    assert_eq!(
        manual_collision.source.column,
        u32::try_from(manual_xml.find("<collision").unwrap() + 1).unwrap()
    );
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
    assert!(collision.pose.relative_to.is_empty());
    assert_eq!(collision.density(), collision.density.unwrap_or(1000.));
    let default = Inertial::default();
    let old = &manual.models[0].links[0].inertial;
    assert_eq!(old.mass, default.mass);
    assert_eq!(old.inertia, default.inertia);
    assert_eq!(old.pose, default.pose);
    assert_eq!(old.fluid_added_mass, default.fluid_added_mass);
    let native: Value =
        serde_json::from_str(include_str!("../auto_default_pose_attributes_native.json")).unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        24,
        "An empty or incomplete native fixture cannot qualify"
    );
    let row = rows.iter().find(|row| row["name"] == name).unwrap();
    if name == "cylinder_old_degrees"
        || name == "cylinder_old_format"
        || name == "cylinder_zero"
        || name == "cylinder_both_false"
    {
        let prior: Value =
            serde_json::from_str(include_str!("../auto_cylinder_inertial_native.json")).unwrap();
        let original = prior["observations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == "retained_negative")
            .unwrap();
        assert!(!original["stderr"].as_str().unwrap().is_empty());
        assert_eq!(row["stderr"], original["stderr"]);
        assert_eq!(collision.density, None);
    } else {
        assert_eq!(row["stderr"].as_str().unwrap(), "", "{name}");
    }
    for key in ["load_errors", "collision_errors", "resolve_errors"] {
        assert!(row[key].as_array().unwrap().is_empty(), "{name}: {key}");
    }
    assert_eq!(row["root_status"], serde_json::json!([[1, 1], [1, 1]]));
    assert_eq!(row["collision_status"], serde_json::json!([1, 1]));
    assert_eq!(row["root_values"][0], row["root_values"][1]);
    let values = row["root_values"][1].as_array().unwrap();
    assert_eq!(values.len(), 18);
    assert_eq!(&values[..2], &[Value::from(1), Value::from(1)]);
    assert_eq!(&values[16..], &[Value::from(0), Value::from(0)]);
    for (actual, native) in std::iter::once(link.inertial.mass)
        .chain(link.inertial.inertia)
        .chain(link.inertial.pose.transform.translation)
        .chain(link.inertial.pose.transform.rotation)
        .zip(&values[2..16])
    {
        four_ulps(actual, native.as_u64().unwrap());
    }
    assert!(link.inertial.pose.relative_to.is_empty());
    assert!(link.inertial.fluid_added_mass.is_none());
    let direct = row["collision_values"].as_array().unwrap();
    assert_eq!(direct.len(), 14);
    assert_eq!(values[2], direct[0]);
    for (actual, native) in collision
        .pose
        .transform
        .translation
        .into_iter()
        .chain(collision.pose.transform.rotation)
        .zip(&direct[7..14])
    {
        four_ulps(actual, native.as_u64().unwrap());
    }
    let matrix = row["collision_moi"].as_array().unwrap();
    assert_eq!(matrix.len(), 9);
    sdf.validate_non_frame_poses().unwrap();
    let actual_frame = sdf
        .resolve_pose(
            &link.inertial.pose,
            &model.scoped_name,
            &link.scoped_name,
            "::",
        )
        .unwrap();
    let expected_frame = plain
        .resolve_pose(
            &expected.links[0].inertial.pose,
            &expected.scoped_name,
            &expected.links[0].scoped_name,
            "::",
        )
        .unwrap();
    assert_eq!(actual_frame, expected_frame);
    let snapshot = format!("{model:?}");
    let mut copy = model.clone();
    assert_eq!(copy.element(), model.element());
    copy.links[0].collisions[0].pose.transform.translation = [99., 98., 97.];
    copy.links[0].collisions[0].density = Some(7.);
    copy.links[0].source.uri = "edited.sdf".into();
    copy.links[0].inertial.mass = 42.;
    assert_eq!(format!("{model:?}"), snapshot);
}
