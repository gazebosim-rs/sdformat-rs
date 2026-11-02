// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "common/auto_capsule_translation.rs"]
mod translation;
use sdformat::*;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str::<Value>(include_str!("auto_capsule_inertial_cases.json"))
        .unwrap()["cases"].as_array().unwrap().clone()
}

fn original() -> String {
    cases()
        .into_iter()
        .find(|row| row["name"] == "original_dimensions")
        .unwrap()["xml"]
        .as_str()
        .unwrap()
        .into()
}

// Same four-ULP policy as the accepted primitive differential helpers.
fn four_ulps(actual: f64, expected: u64) {
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

fn manual_policy(values: &[u64]) -> Result<Sdf, Diagnostic> {
    let f = |i| f64::from_bits(values[i]).to_string();
    let xml = format!(
        "<sdf version='1.12'><model name='policy'><link name='body'><inertial><mass>{}</mass><inertia><ixx>{}</ixx><iyy>{}</iyy><izz>{}</izz><ixy>{}</ixy><ixz>{}</ixz><iyz>{}</iyz></inertia></inertial></link></model></sdf>",
        f(2),
        f(3),
        f(4),
        f(5),
        f(6),
        f(7),
        f(8)
    );
    load_str(&xml, "existing-manual-policy.sdf", &NoResources)
}

fn owned_values(xml: &str, uri: &str, sdf: &Sdf, density: Option<f64>) {
    assert_eq!(sdf.version, "1.12");
    assert!(sdf.worlds.is_empty());
    assert_eq!(sdf.models.len(), 1);
    let model = &sdf.models[0];
    assert_eq!((&*model.name, &*model.scoped_name), ("m", "m"));
    assert_eq!(model.pose, Pose::default());
    assert!(!(model.is_static || model.self_collide || model.enable_wind));
    assert!(model.allow_auto_disable);
    assert!(model.raw_namespace.is_none());
    assert!(model.canonical_link.is_empty() && model.placement_frame.is_empty());
    assert!(model.include.is_none() && model.merged_includes.is_empty());
    assert!(model.models.is_empty() && model.frames.is_empty() && model.joints.is_empty());
    assert!(model.plugins.is_empty() && model.extensions.is_empty());
    let children = model.element().unwrap().children();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].tag(), "link");
    assert_eq!(children[0].name_attribute(), Some("l"));
    assert_eq!(model.links.len(), 1);
    let link = &model.links[0];
    assert_eq!((&*link.name, &*link.scoped_name), ("l", "m::l"));
    assert_eq!(link.pose, Pose::default());
    assert_eq!(link.properties, LinkProperties::default());
    assert!(link.velocity_decay.is_none());
    assert!(link.visuals.is_empty() && link.sensors.is_empty() && link.lights.is_empty());
    assert!(
        link.particle_emitters.is_empty() && link.projectors.is_empty() && link.plugins.is_empty()
    );
    assert_eq!(link.collisions.len(), 1);
    let collision = &link.collisions[0];
    assert_eq!(collision.name, "c");
    assert!(matches!(collision.geometry, Geometry::Capsule { .. }));
    assert_eq!(collision.pose, Pose::default());
    assert_eq!(collision.density, density);
    assert_eq!(collision.density(), density.unwrap_or(1000.));
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
    assert_eq!(link.inertial.pose, Pose::default());
    assert!(link.inertial.fluid_added_mass.is_none());
    assert_eq!(sdf.pose_of("m::l", "::").unwrap(), Transform::default());
    assert_eq!(
        sdf.resolve_pose(&link.inertial.pose, "m", "m::l", "::")
            .unwrap(),
        Transform::default()
    );
    sdf.validate_non_frame_poses().unwrap();
    let mut copy = model.clone();
    assert_eq!(copy.element(), model.element());
    let mass = link.inertial.mass;
    let geometry = collision.geometry.clone();
    copy.links[0].inertial.mass = 42.;
    copy.links[0].collisions[0].geometry = Geometry::Empty;
    copy.links[0].collisions[0].density = Some(7.);
    assert_eq!(link.inertial.mass, mass);
    assert_eq!(collision.geometry, geometry);
    assert_eq!(collision.density, density);
}

#[test]
fn actual_capsule_corpus_preserves_values_or_explicit_numeric_and_pose_refusals() {
    let cases = cases();
    let native: Value =
        serde_json::from_str(include_str!("auto_capsule_inertial_native.json")).unwrap();
    let rows = native["observations"].as_array().unwrap();
    assert_eq!(cases.len(), 24);
    assert_eq!(
        rows.len(),
        cases.len(),
        "Every actual native row is required; empty data cannot qualify"
    );
    for (case, row) in cases.iter().zip(rows) {
        assert_eq!(case["name"], row["name"]);
        assert_eq!(row["root_values"].as_array().unwrap().len(), 2);
        assert_eq!(row["root_status"].as_array().unwrap().len(), 2);
        assert_eq!(row["collision_values"].as_array().unwrap().len(), 14);
        assert_eq!(row["collision_moi"].as_array().unwrap().len(), 9);
        let native_errors = if matches!(
            case["name"].as_str().unwrap(),
            "maximum_density" | "native_density_prefix_overflow" | "maximum_length"
        ) {
            serde_json::json!([19])
        } else {
            serde_json::json!([])
        };
        assert_eq!(row["load_errors"], native_errors);
        assert_eq!(row["collision_errors"], native_errors);
        assert!(row["resolve_errors"].as_array().unwrap().is_empty());
        // Valid flags also describe retained defaults and the zero-mass result.
        assert_eq!(row["root_status"], serde_json::json!([[1, 1], [1, 1]]));
        assert_eq!(row["collision_status"], serde_json::json!([1, 1]));
        assert_eq!(row["root_values"][0], row["root_values"][1]);
        let warns = matches!(
            case["name"].as_str().unwrap(),
            "negative_radius"
                | "zero_length"
                | "both_nonpositive"
                | "missing_density"
                | "translated"
                | "rotated"
                | "retained_mixed_refusal"
        );
        assert_eq!(!row["stderr"].as_str().unwrap().is_empty(), warns);
        if case["name"] == "minimum_radius" {
            assert!(
                row["root_values"][1].as_array().unwrap()[2..9]
                    .iter()
                    .all(|value| value.as_u64() == Some(0))
            );
        }
        let xml = case["xml"].as_str().unwrap();
        let uri = case["uri"].as_str().unwrap();
        let result = load_str(xml, uri, &NoResources);
        if case["name"] == "translated" {
            translation::assert_translated_input(xml, uri, "old_translated_corpus");
            continue;
        }
        if case["rust_error"] == "Unsupported" {
            let error = result.unwrap_err();
            assert_eq!(error.code, ErrorCode::Unsupported, "{}", case["name"]);
            assert_eq!(error.source.uri, uri);
            continue;
        }
        assert!(row["load_errors"].as_array().unwrap().is_empty());
        assert!(row["resolve_errors"].as_array().unwrap().is_empty());
        assert!(row["collision_errors"].as_array().unwrap().is_empty());
        assert_eq!(row["root_status"], serde_json::json!([[1, 1], [1, 1]]));
        assert_eq!(row["collision_status"], serde_json::json!([1, 1]));
        assert_eq!(row["root_values"][0], row["root_values"][1]);
        let values: Vec<u64> = row["root_values"][1]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap())
            .collect();
        assert_eq!(values.len(), 18);
        assert_eq!(&values[..2], &[1, 1]);
        assert_eq!(&values[16..], &[0, 0]);
        if let Err(error) = manual_policy(&values) {
            assert_eq!(error.code, ErrorCode::InvalidValue);
            assert_eq!(result.unwrap_err().code, ErrorCode::InvalidValue);
            continue;
        }
        let sdf = result.unwrap_or_else(|error| panic!("{}: {error}", case["name"]));
        let link = &sdf.models[0].links[0];
        for (actual, native) in std::iter::once(link.inertial.mass)
            .chain(link.inertial.inertia)
            .chain(link.inertial.pose.transform.translation)
            .chain(link.inertial.pose.transform.rotation)
            .zip(&values[2..16])
        {
            four_ulps(actual, *native);
        }
        for (actual, index) in link.inertial.inertia.into_iter().zip([0, 4, 8, 1, 2, 5]) {
            four_ulps(actual, row["collision_moi"][index].as_u64().unwrap());
        }
        let density = case["density"].as_str().map(|s| {
            if s.trim().is_empty() {
                1000.
            } else {
                s.parse::<f64>().unwrap()
            }
        });
        owned_values(xml, uri, &sdf, density);
        let manual = load_str(
            &xml.replace("auto='true'", "auto='false'"),
            uri,
            &NoResources,
        )
        .unwrap();
        let old = &manual.models[0].links[0];
        let default = Inertial::default();
        assert_eq!(old.inertial.mass, default.mass);
        assert_eq!(old.inertial.inertia, default.inertia);
        assert_eq!(old.inertial.pose, default.pose);
        assert_eq!(old.inertial.fluid_added_mass, default.fluid_added_mass);
        assert_eq!(old.collisions[0].geometry, link.collisions[0].geometry);
        assert_eq!(old.collisions[0].pose, link.collisions[0].pose);
        assert_eq!(old.collisions[0].density, density);
        if case["name"] == "original_dimensions" {
            assert_eq!(
                link.collisions[0].geometry,
                Geometry::Capsule {
                    radius: 0.1,
                    length: 2.
                }
            );
        }
    }
}

#[test]
fn capsule_pose_attribute_override_and_multiplicity_domains_remain_closed() {
    let original = original();
    for pose in [
        "<pose>1 0 0 0 0 0</pose>",
        "<pose>0 0 0 0.1 0 0</pose>",
        "<pose>1 0 0 0.1 0 0</pose>",
        "<pose relative_to=''>0 0 0 0 0 0</pose>",
        "<pose relative_to='l'>0 0 0 0 0 0</pose>",
        "<pose degrees='false'>0 0 0 0 0 0</pose>",
        "<pose rotation_format='euler_rpy'>0 0 0 0 0 0</pose>",
        "<pose rotation_format='quat_xyzw'>0 0 0 0 0 0 1</pose>",
    ] {
        let xml = original.replace("<pose>0 0 0 0 0 0</pose>", pose);
        if pose == "<pose>1 0 0 0 0 0</pose>" {
            translation::assert_translated_input(
                &xml,
                "closed-capsule.sdf",
                "old_translated_closed",
            );
        } else {
            let error = load_str(&xml, "closed-capsule.sdf", &NoResources).unwrap_err();
            assert_eq!(error.code, ErrorCode::Unsupported);
            assert_eq!(error.source.uri, "closed-capsule.sdf");
        }
    }
    for override_xml in [
        "<mass>1</mass>",
        "<pose/>",
        "<inertia/>",
        "<density>1</density>",
    ] {
        let xml = original.replace(
            "<inertial auto='true'/>",
            &format!("<inertial auto='true'>{override_xml}</inertial>"),
        );
        assert_eq!(
            load_str(&xml, "override.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    for density in ["0", "-1", "NaN", "inf"] {
        let xml = original.replace(
            "<density>2710</density>",
            &format!("<density>{density}</density>"),
        );
        assert_eq!(
            load_str(&xml, "density.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::InvalidValue
        );
    }
    let multiple = original.replace(
        "</link>",
        "<collision name='second'><geometry><capsule/></geometry></collision></link>",
    );
    assert_eq!(
        load_str(&multiple, "multiple.sdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Unsupported
    );
}

#[test]
fn origin_capsule_include_and_clone_preserve_source_and_local_inertial_frame() {
    struct Resources;
    impl ResourceResolver for Resources {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!((uri, parent), ("capsule", "world.sdf"));
            Ok(Resource {
                canonical_uri: "canonical://capsule.sdf".into(),
                xml: original(),
            })
        }
    }
    let sdf=load_str("<sdf version='1.12'><world name='w'><include><uri>capsule</uri><name>imported</name><pose>4 5 6 0 0 0</pose></include></world></sdf>","world.sdf",&Resources).unwrap();
    let model = &sdf.worlds[0].models[0];
    let link = &model.links[0];
    assert_eq!(link.inertial.pose, Pose::default());
    assert_eq!(link.collisions[0].pose, Pose::default());
    assert_eq!(
        link.collisions[0].geometry,
        Geometry::Capsule {
            radius: 0.1,
            length: 2.
        }
    );
    assert_eq!(link.collisions[0].density, Some(2710.));
    assert_eq!(
        sdf.resolve_pose(&link.inertial.pose, "w::imported", "w::imported::l", "::")
            .unwrap()
            .translation,
        [4., 5., 6.]
    );
    for source in [&model.source, &link.source, &link.collisions[0].source] {
        assert_eq!(source.uri, "canonical://capsule.sdf");
        assert!(source.line > 0 && source.column > 0);
    }
    let mass = link.inertial.mass;
    let mut copy = model.clone();
    assert_eq!(copy.element(), model.element());
    copy.links[0].inertial.mass = 42.;
    copy.links[0].collisions[0].density = None;
    copy.links[0].collisions[0].geometry = Geometry::Empty;
    assert_eq!(link.inertial.mass, mass);
    assert_eq!(link.collisions[0].density, Some(2710.));
    assert_eq!(
        link.collisions[0].geometry,
        Geometry::Capsule {
            radius: 0.1,
            length: 2.
        }
    );
    sdf.validate_non_frame_poses().unwrap();
}
