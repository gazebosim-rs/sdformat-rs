// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
use serde_json::Value;
#[path = "support/world_environment_projection.rs"]
mod environment;
#[path = "support/world_gui_projection.rs"]
mod gui_projection;
use gui_projection::light as projection;
#[path = "support/world_scene_projection.rs"]
mod scene_projection;
fn data() -> Value {
    serde_json::from_str(include_str!("document_lights.json")).unwrap()
}
fn tokens(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().into())
        .collect()
}
fn first_light(sdf: &Sdf) -> &Light {
    if let Some(world) = sdf.worlds.first() {
        if let Some(light) = world.lights.first() {
            return light;
        }
        return &world.models[0].links[0].lights[0];
    }
    &sdf.models[0].links[0].lights[0]
}
#[test]
fn native_schema_fields_preserve_defaults_clamps_and_type_specific_getters() {
    let data = data();
    let rows = data["fields"].as_array().unwrap();
    assert_eq!(rows.len(), 42);
    let mut accepted = 0;
    for row in rows {
        let xml = format!(
            "<sdf version='1.12'><world name='w'>{}</world></sdf>",
            row["xml"].as_str().unwrap()
        );
        let expected = tokens(&row["native"]["lines"][0]);
        let result = load_str(&xml, "light-fields.sdf", &NoResources);
        if expected == ["REJECT"] {
            assert!(result.is_err(), "{}", row["id"]);
            continue;
        }
        let sdf = result.unwrap_or_else(|e| panic!("{} {e:?}", row["id"]));
        projection::compare(&projection::projection(first_light(&sdf)), &expected);
        accepted += 1;
    }
    assert_eq!(accepted, 30);
}
#[test]
fn typed_documents_have_native_values_and_actual_declaration_origins() {
    let data = data();
    let rows = data["typed_documents"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    for (row, column) in rows.iter().zip([37, 52]) {
        let sdf = load_str(
            row["xml"].as_str().unwrap(),
            "typed-light.sdf",
            &NoResources,
        )
        .unwrap();
        let light = first_light(&sdf);
        projection::compare(
            &projection::projection(light),
            &tokens(&row["native"]["lines"][0]),
        );
        assert_eq!(
            light.source,
            Some(Source {
                uri: "typed-light.sdf".into(),
                line: 1,
                column
            })
        );
        let (scope, frame) = if sdf.worlds.is_empty() {
            ("m", "m::l")
        } else {
            ("w", "w")
        };
        projection::pose(
            sdf.resolve_pose(&light.pose, scope, frame, frame).unwrap(),
            &tokens(&row["native"]["lines"][1])[3..],
        );
        sdf.validate_non_frame_poses().unwrap();
    }
}
#[test]
fn namespaces_duplicates_and_deferred_pose_errors_follow_native_controls() {
    for row in data()["document_shapes"].as_array().unwrap() {
        let expected = tokens(&row["native"]["lines"][0]);
        let value = load_str(
            row["xml"].as_str().unwrap(),
            "shape-light.sdf",
            &NoResources,
        );
        if expected == ["REJECT"] {
            assert_eq!(value.unwrap_err().code, ErrorCode::DuplicateName);
            continue;
        }
        let sdf = value.unwrap();
        let light = first_light(&sdf);
        projection::compare(&projection::projection(light), &expected);
        if row["id"].as_str().unwrap().contains("deferred") {
            assert_eq!(
                tokens(&row["native"]["lines"][1]),
                ["POSE", "default", "REJECT"]
            );
            assert_eq!(
                sdf.validate_non_frame_poses().unwrap_err().code,
                ErrorCode::UnknownReference
            );
        } else {
            sdf.validate_non_frame_poses().unwrap();
        }
    }
}
#[test]
fn nonzero_link_pose_keeps_native_default_target_and_local_scope() {
    let data = data();
    let row = &data["offset_document"];
    let sdf = load_str(
        row["xml"].as_str().unwrap(),
        "offset-light.sdf",
        &NoResources,
    )
    .unwrap();
    let light = first_light(&sdf);
    projection::compare(
        &projection::projection(light),
        &tokens(&row["native"]["lines"][0]),
    );
    let targets = ["w::m::f", "w::m", "w::m::world", "w::m::l", "w::m::missing"];
    for (line, target) in row["native"]["lines"].as_array().unwrap()[1..]
        .iter()
        .zip(targets)
    {
        let expected = tokens(line);
        let result = sdf.resolve_pose(&light.pose, "w::m", "w::m::l", target);
        if expected[2] == "REJECT" {
            assert_eq!(expected.len(), 3);
            assert_eq!(result.unwrap_err().code, ErrorCode::UnknownReference);
        } else {
            assert_eq!(expected.len(), 10);
            projection::pose(result.unwrap(), &expected[3..]);
        }
    }
}
#[test]
fn document_schema_defaults_do_not_hide_unknown_payloads() {
    for payload in [
        "<diffuse bad='x'/>",
        "<diffuse><unknown/></diffuse>",
        "<specular bad='x'/>",
        "<attenuation><range bad='x'/></attenuation>",
        "<spot><inner_angle><unknown/></inner_angle></spot>",
        "<direction bad='x'>1 2 3</direction>",
        "<direction><unknown/></direction>",
        "<plugin name='p' filename='f'/>",
    ] {
        let xml = format!(
            "<sdf version='1.12'><world name='w'><light name='n' type='point'>{payload}</light></world></sdf>"
        );
        assert_eq!(
            load_str(&xml, "payload.sdf", &NoResources)
                .unwrap_err()
                .code,
            ErrorCode::Unsupported
        );
    }
    for payload in ["<scene/>", "<gui/>", "<projector/>"] {
        let xml = format!(
            "<sdf version='1.12'><world name='w'>{payload}<light name='n' type='point'/></world></sdf>"
        );
        if payload == "<scene/>" {
            let sdf = load_str(&xml, "payload.sdf", &NoResources).unwrap();
            scene_projection::assert_scene(&sdf, &xml, "payload.sdf", false, 1);
            let corpus: serde_json::Value =
                serde_json::from_str(include_str!("world_scene.json")).unwrap();
            let row = corpus["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["name"] == "old_scene_light")
                .unwrap();
            assert_eq!(row["xml"].as_str().unwrap(), xml);
            let w = &sdf.worlds[0];
            let native: Vec<Vec<String>> = row["light_native"]["stdout"]
                .as_str()
                .unwrap()
                .lines()
                .map(|line| line.split_whitespace().map(str::to_owned).collect())
                .collect();
            assert_eq!(native.len(), 2);
            projection::compare(&projection::projection(&w.lights[0]), &native[0]);
            assert_eq!(&native[1][..3], ["POSE", "default", "OK"]);
            projection::pose(
                sdf.resolve_pose(&w.lights[0].pose, "w", "w", "w").unwrap(),
                &native[1][3..],
            );
            assert_eq!(w.lights[0].source.as_ref().unwrap().uri, "payload.sdf");
            sdf.validate_non_frame_poses().unwrap();
        } else if payload == "<gui/>" {
            let sdf = load_str(&xml, "payload.sdf", &NoResources).unwrap();
            let corpus: Value = serde_json::from_str(include_str!("world_gui.json")).unwrap();
            let row = corpus["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["name"] == "old_light")
                .unwrap();
            assert_eq!(row["xml"].as_str().unwrap(), xml);
            gui_projection::assert_gui(
                &sdf,
                &xml,
                "payload.sdf",
                "1.12",
                false,
                row["light_native"]["stdout"].as_str(),
            );
        } else {
            assert_eq!(
                load_str(&xml, "payload.sdf", &NoResources)
                    .unwrap_err()
                    .code,
                ErrorCode::Unsupported
            );
        }
    }

    let data: Value = serde_json::from_str(include_str!("world_environment.json")).unwrap();
    let row = data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "prepared-old-wind")
        .unwrap();
    assert_eq!(
        row["xml"],
        "<sdf version='1.12'><world name='w'><wind/><light name='n' type='point'/></world></sdf>"
    );
    let sdf = load_str(row["xml"].as_str().unwrap(), "payload.sdf", &NoResources).unwrap();
    assert_eq!(sdf.worlds.len(), 1);
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "w");
    environment::compare(world, &row["native"]);
    assert_eq!(world.models.len(), 0);
    assert_eq!(world.lights.len(), 1);
    let native: Vec<Vec<String>> = row["light_native"]["stdout"]
        .as_str()
        .unwrap()
        .lines()
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .collect();
    assert_eq!(native.len(), 2);
    projection::compare(&projection::projection(&world.lights[0]), &native[0]);
    assert_eq!(&native[1][..3], ["POSE", "default", "OK"]);
    projection::pose(
        sdf.resolve_pose(&world.lights[0].pose, "w", "w", "w")
            .unwrap(),
        &native[1][3..],
    );
    sdf.validate_non_frame_poses().unwrap();
}
