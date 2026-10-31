/*
 * Copyright 2012 Open Source Robotics Foundation
 * Copyright 2023 Open Source Robotics Foundation
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
use sdformat::{ErrorCode, Geometry, Model, Sdf, Transform, load_urdf_str};
use serde_json::Value;

const ORIGINAL: &str = include_str!("fixtures/urdf_single_link_original.urdf");
fn data() -> Value {
    serde_json::from_str(include_str!("urdf_single_link.json")).unwrap()
}
fn hex(value: &str) -> String {
    if value.is_empty() {
        return "-".into();
    }
    value
        .as_bytes()
        .iter()
        .map(|v| format!("{v:02x}"))
        .collect()
}
fn numbers(values: impl IntoIterator<Item = f64>) -> String {
    values
        .into_iter()
        .map(|v| format!(" {}", v.to_bits()))
        .collect()
}
fn pose(value: &Transform) -> String {
    numbers(value.translation.into_iter().chain(value.rotation))
}
fn shape(value: &Geometry) -> String {
    match value {
        Geometry::Sphere { radius } => format!("SPHERE{}", numbers([*radius])),
        Geometry::Box { size } => format!("BOX{}", numbers(*size)),
        Geometry::Cylinder { radius, length } => format!("CYLINDER{}", numbers([*radius, *length])),
        _ => panic!("unadmitted geometry"),
    }
}
fn stream(sdf: &Sdf, model: &Model) -> String {
    let mut result = format!(
        "ERRORS 0\nROOT {}\nMODEL {} {} {}{}\n",
        hex(&sdf.version),
        hex(&model.name),
        model.links.len(),
        model.joints.len(),
        pose(&model.pose.transform)
    );
    for link in &model.links {
        result += &format!(
            "LINK {}{} {} {}\nINERTIAL{}{}\nRESOLVED link 0{}\n",
            hex(&link.name),
            pose(&link.pose.transform),
            link.visuals.len(),
            link.collisions.len(),
            numbers(
                [link.inertial.mass]
                    .into_iter()
                    .chain(link.inertial.inertia)
            ),
            pose(&link.inertial.pose.transform),
            pose(&sdf.pose_of(&link.scoped_name, &model.scoped_name).unwrap())
        );
        for visual in &link.visuals {
            result += &format!(
                "VISUAL {}{} {}\n",
                hex(&visual.name),
                pose(&visual.pose.transform),
                shape(visual.geometry.as_ref().unwrap())
            );
            if let Some(material) = &visual.material {
                let script = material.script.as_ref().unwrap();
                result += &format!("MATERIAL {} {}\n", hex(&script.name), hex(&script.uris[0]));
            }
            result += &format!(
                "RESOLVED visual 0{}\n",
                pose(
                    &sdf.resolve_pose(
                        &visual.pose,
                        &model.scoped_name,
                        &link.scoped_name,
                        &model.scoped_name
                    )
                    .unwrap()
                )
            );
        }
        for collision in &link.collisions {
            result += &format!(
                "COLLISION {}{} {}\nMU{}\nRESOLVED collision 0{}\n",
                hex(&collision.name),
                pose(&collision.pose.transform),
                shape(&collision.geometry),
                numbers([
                    collision.surface.friction.ode.mu,
                    collision.surface.friction.ode.mu2
                ]),
                pose(
                    &sdf.resolve_pose(
                        &collision.pose,
                        &model.scoped_name,
                        &link.scoped_name,
                        &model.scoped_name
                    )
                    .unwrap()
                )
            );
        }
    }
    result
}
fn compare(actual: &str, expected: &str, name: &str) {
    let actual: Vec<_> = actual
        .lines()
        .map(|s| s.split_whitespace().collect::<Vec<_>>())
        .collect();
    let expected: Vec<_> = expected
        .lines()
        .map(|s| s.split_whitespace().collect::<Vec<_>>())
        .collect();
    assert_eq!(actual.len(), expected.len(), "{name}");
    for (actual, expected) in actual.iter().zip(&expected) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "{name}: {actual:?} {expected:?}"
        );
        let start = match actual[0] {
            "MODEL" => Some(4),
            "LINK" | "VISUAL" | "COLLISION" => Some(2),
            "INERTIAL" => Some(8),
            "RESOLVED" => Some(3),
            _ => None,
        };
        for i in 0..actual.len() {
            if start.is_some_and(|s| (s + 3..s + 7).contains(&i)) {
                continue;
            }
            assert_eq!(actual[i], expected[i], "{name} field{i}");
        }
        if let Some(start) = start {
            let a: Vec<_> = actual[start + 3..start + 7]
                .iter()
                .map(|s| f64::from_bits(s.parse().unwrap()))
                .collect();
            let b: Vec<_> = expected[start + 3..start + 7]
                .iter()
                .map(|s| f64::from_bits(s.parse().unwrap()))
                .collect();
            let same = a.iter().zip(&b).all(|(a, b)| (a - b).abs() <= 1e-12);
            let opposite = a.iter().zip(&b).all(|(a, b)| (a + b).abs() <= 1e-12);
            assert!(same || opposite, "{name} quaternion {a:?} {b:?}");
        }
    }
}
#[test]
fn native_single_link_import_values_names_units_and_strict_boundaries() {
    for row in data()["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let result = load_urdf_str(row["xml"].as_str().unwrap(), "input.urdf");
        match row["rust_policy"].as_str().unwrap() {
            "accepted" => {
                let import = result.unwrap_or_else(|e| panic!("{name}: {e:?}"));
                assert_eq!(import.sdf.models.len(), 1);
                assert!(import.sdf.worlds.is_empty());
                let model = &import.sdf.models[0];
                assert!(
                    !model.is_static
                        && !model.self_collide
                        && model.allow_auto_disable
                        && !model.enable_wind
                );
                compare(
                    &stream(&import.sdf, model),
                    row["stdout"].as_str().unwrap(),
                    name,
                );
                import.sdf.validate_non_frame_poses().unwrap();
            }
            policy => {
                let error = result.expect_err(name);
                let code = match policy {
                    "InvalidValue" => ErrorCode::InvalidValue,
                    "Unsupported" => ErrorCode::Unsupported,
                    "Missing" => ErrorCode::Missing,
                    "UnknownReference" => ErrorCode::UnknownReference,
                    "Version" => ErrorCode::Version,
                    _ => panic!("{policy}"),
                };
                assert_eq!(error.code, code, "{name}: {error:?}");
                assert_eq!(error.source.uri, "input.urdf");
            }
        }
    }
}
#[test]
fn original_valid_basic_urdf_loads_model_and_named_link() {
    let xml = include_str!("fixtures/urdf_basic_original.urdf");
    let import = load_urdf_str(xml, "basic.urdf").unwrap();
    let model = import.sdf.models.first().unwrap();
    assert!(model.links.iter().any(|link| link.name == "link1"));
    assert_eq!(model.links.len(), 1);
    assert_eq!(model.links[0].inertial.mass, 0.1);
}
fn text_at(xml: &str, path: &[&str]) -> Option<String> {
    use quick_xml::{Reader, events::Event};
    let mut reader = Reader::from_str(xml);
    let mut stack = Vec::new();
    let mut found = None;
    loop {
        match reader.read_event().unwrap() {
            Event::Start(value) => {
                stack.push(
                    std::str::from_utf8(value.name().as_ref())
                        .unwrap()
                        .to_owned(),
                );
                if stack == path {
                    found = Some(String::new());
                }
            }
            Event::Empty(value) => {
                let mut here = stack.clone();
                here.push(
                    std::str::from_utf8(value.name().as_ref())
                        .unwrap()
                        .to_owned(),
                );
                if here == path {
                    found = Some(String::new());
                }
            }
            Event::End(_) => {
                stack.pop();
            }
            Event::Text(value) if stack == path => {
                found.as_mut().unwrap().push_str(&value.decode().unwrap());
            }
            Event::Eof => break,
            _ => {}
        }
    }
    found
}
#[test]
fn original_parse_whitespace_preserves_converted_tree_and_exact_three_text_values() {
    let import = load_urdf_str(ORIGINAL, "whitespace.urdf").unwrap();
    assert_eq!(import.original_xml(), ORIGINAL);
    let xml = import.converted_xml();
    for path in [
        "sdf",
        "sdf/model",
        "sdf/model/link",
        "sdf/model/link/visual",
        "sdf/model/link/collision",
        "sdf/model/link/visual/material",
        "sdf/model/link/visual/material/script",
        "sdf/model/link/visual/material/script/name",
        "sdf/model/link/collision/surface",
        "sdf/model/link/collision/surface/friction",
        "sdf/model/link/collision/surface/friction/ode",
        "sdf/model/link/collision/surface/friction/ode/mu",
        "sdf/model/link/collision/surface/friction/ode/mu2",
    ] {
        assert!(
            text_at(xml, &path.split('/').collect::<Vec<_>>()).is_some(),
            "{path}"
        );
    }
    assert_eq!(
        text_at(
            xml,
            &[
                "sdf", "model", "link", "visual", "material", "script", "name"
            ]
        )
        .as_deref(),
        Some("Gazebo/Orange")
    );
    assert_eq!(
        text_at(
            xml,
            &[
                "sdf",
                "model",
                "link",
                "collision",
                "surface",
                "friction",
                "ode",
                "mu"
            ]
        )
        .as_deref(),
        Some("100")
    );
    assert_eq!(
        text_at(
            xml,
            &[
                "sdf",
                "model",
                "link",
                "collision",
                "surface",
                "friction",
                "ode",
                "mu2"
            ]
        )
        .as_deref(),
        Some("1000")
    );
}
#[test]
fn immutable_original_and_converted_snapshots_have_distinct_honest_sources() {
    let mut import = {
        let xml = ORIGINAL.to_owned();
        load_urdf_str(&xml, "original.urdf").unwrap()
    };
    assert_eq!(import.original_source().uri, "original.urdf");
    assert_eq!(
        (
            import.original_source().line,
            import.original_source().column
        ),
        (1, 1)
    );
    assert_eq!(
        import.converted_source().uri,
        "generated:urdf-to-sdf:original.urdf"
    );
    let generated = import.converted_xml().to_owned();
    let model = &mut import.sdf.models[0];
    assert_eq!(model.source.uri, "generated:urdf-to-sdf:original.urdf");
    assert_eq!(model.source.line, 1);
    assert_eq!(
        model.source.column as usize,
        generated.find("<model").unwrap() + 1
    );
    let element = model.loaded_element.clone().unwrap();
    assert!(element.has_child("link"));
    let link = &mut model.links[0];
    assert_eq!(link.source.uri, model.source.uri);
    assert_eq!(
        link.source.column as usize,
        generated.find("<link").unwrap() + 1
    );
    assert_eq!(link.inertial.pose.relative_to, "");
    assert_eq!(link.visuals[0].source.uri, model.source.uri);
    assert_eq!(link.collisions[0].source.uri, model.source.uri);
    assert_eq!(
        link.collisions[0].surface.source.as_ref().unwrap().uri,
        model.source.uri
    );
    model.name = "edited".into();
    link.inertial.mass = 2.;
    assert_eq!(model.loaded_element.as_ref(), Some(&element));
    assert_eq!(import.original_xml(), ORIGINAL);
    assert_eq!(import.converted_xml(), generated);
    let mut copied = import.original_source().clone();
    copied.uri.clear();
    assert_eq!(import.original_source().uri, "original.urdf");
}
#[test]
fn unknown_elements_and_attributes_are_rejected_before_dependency_parsing() {
    for tag in [
        "robot",
        "link",
        "inertial",
        "mass",
        "inertia",
        "origin",
        "visual",
        "collision",
        "geometry",
        "sphere",
        "gazebo",
        "material",
        "mu1",
        "mu2",
    ] {
        let start = format!("<{tag}");
        let at = ORIGINAL
            .match_indices(&start)
            .find(|(i, _)| matches!(ORIGINAL.as_bytes()[i + start.len()], b' ' | b'>' | b'/'))
            .unwrap()
            .0
            + start.len();
        let mut xml = ORIGINAL.to_owned();
        xml.insert_str(at, " unsupported='value'");
        let error = load_urdf_str(&xml, "unknown.urdf").unwrap_err();
        assert_eq!(error.code, ErrorCode::Unsupported, "{tag}: {error:?}");
        assert_eq!(error.source.uri, "unknown.urdf");
    }
    let xml = ORIGINAL
        .replace("value=\"0.1\"", "value=\"not-a-number\"")
        .replace("</robot>", "<unknown/></robot>");
    assert_eq!(
        load_urdf_str(&xml, "precedence.urdf").unwrap_err().code,
        ErrorCode::Unsupported
    );
}
#[test]
fn bounded_import_rejects_missing_nested_and_unapplied_metadata_explicitly() {
    for (xml, code) in [
        ("<sdf version='1.12'/>", ErrorCode::Xml),
        ("<robot><link name='l'/></robot>", ErrorCode::Missing),
        ("<robot name='r'/>", ErrorCode::Unsupported),
        ("<!DOCTYPE robot><robot name='r'/>", ErrorCode::Unsupported),
    ] {
        assert_eq!(load_urdf_str(xml, "invalid.urdf").unwrap_err().code, code);
    }
    for (from, to, code) in [
        ("<mu1>", "<mu1><unknown/>", ErrorCode::Unsupported),
        ("<material>", "<material extra='x'>", ErrorCode::Unsupported),
        ("Gazebo/Orange", "", ErrorCode::Missing),
        ("1000", "", ErrorCode::Missing),
        ("<sphere radius=\"1.0\"/>", "<sphere/>", ErrorCode::Missing),
        ("<geometry>", "<geometry><unknown/>", ErrorCode::Unsupported),
    ] {
        let xml = ORIGINAL.replacen(from, to, 1);
        assert_eq!(
            load_urdf_str(&xml, "invalid.urdf").unwrap_err().code,
            code,
            "{from}"
        );
    }
}

#[test]
fn every_admitted_node_rejects_unknown_children() {
    for tag in [
        "robot",
        "link",
        "inertial",
        "mass",
        "inertia",
        "origin",
        "visual",
        "collision",
        "geometry",
        "sphere",
        "gazebo",
        "material",
        "mu1",
        "mu2",
    ] {
        let marker = format!("<{tag}");
        let start = ORIGINAL
            .match_indices(&marker)
            .find(|(i, _)| matches!(ORIGINAL.as_bytes()[i + marker.len()], b' ' | b'>' | b'/'))
            .unwrap()
            .0;
        let end = start + ORIGINAL[start..].find('>').unwrap();
        let mut xml = ORIGINAL.to_owned();
        if ORIGINAL.as_bytes()[end - 1] == b'/' {
            let replacement = format!("{}><unknown/></{tag}>", &ORIGINAL[start..end - 1]);
            xml.replace_range(start..end + 1, &replacement);
        } else {
            xml.insert_str(end + 1, "<unknown/>");
        }
        let error = load_urdf_str(&xml, "unknown-child.urdf").unwrap_err();
        assert_eq!(error.code, ErrorCode::Unsupported, "{tag}: {error:?}");
        assert_eq!(error.source.uri, "unknown-child.urdf");
    }
}
#[test]
fn single_link_targets_and_required_shape_are_not_silently_discarded() {
    let data = data();
    let base = data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "inertial_only")
        .unwrap()["xml"]
        .as_str()
        .unwrap();
    for extra in [
        "<gazebo reference='base'><material>Gazebo/Orange</material></gazebo>",
        "<gazebo reference='base'><mu1>1</mu1></gazebo>",
        "<gazebo reference='base'/><gazebo reference='base'/>",
    ] {
        let xml = base.replace("</robot>", &format!("{extra}</robot>"));
        assert_eq!(
            load_urdf_str(&xml, "targets.urdf").unwrap_err().code,
            ErrorCode::Unsupported
        );
    }
    for (from, to, code) in [
        ("name=\"base\"", "", ErrorCode::Missing),
        ("ixx=\"1\"", "", ErrorCode::Missing),
        (
            "<inertial>",
            "<inertial><origin/><origin/>",
            ErrorCode::InvalidValue,
        ),
        (
            "<inertial>",
            "<inertial><origin rpy='invalid'/>",
            ErrorCode::InvalidValue,
        ),
    ] {
        let xml = base.replace(from, to);
        assert_eq!(
            load_urdf_str(&xml, "required.urdf").unwrap_err().code,
            code,
            "{from}"
        );
    }
    for geometry in [
        "<geometry/>",
        "<geometry><sphere radius='1'/><box size='1 1 1'/></geometry>",
    ] {
        let xml = base.replace("</link>", &format!("<visual>{geometry}</visual></link>"));
        assert_eq!(
            load_urdf_str(&xml, "shape.urdf").unwrap_err().code,
            ErrorCode::InvalidValue
        );
    }
}
#[test]
fn sdf_loader_and_includes_keep_their_existing_explicit_format_boundary() {
    use sdformat::{NoResources, Resource, ResourceResolver, load_str};
    assert_eq!(
        load_str(ORIGINAL, "original.urdf", &NoResources)
            .unwrap_err()
            .code,
        ErrorCode::Xml
    );
    struct Urdf;
    impl ResourceResolver for Urdf {
        fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
            assert_eq!(uri, "robot.urdf");
            assert_eq!(parent, "world.sdf");
            Ok(Resource {
                canonical_uri: "resolved://robot.urdf".into(),
                xml: ORIGINAL.into(),
            })
        }
    }
    let sdf = "<sdf version='1.12'><world name='w'><include><uri>robot.urdf</uri></include></world></sdf>";
    let error = load_str(sdf, "world.sdf", &Urdf).unwrap_err();
    assert_eq!(error.code, ErrorCode::Xml);
    assert_eq!(error.source.uri, "resolved://robot.urdf");
}

#[test]
fn missing_visual_and_collision_geometry_report_the_original_node() {
    let data = data();
    let base = data["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "inertial_only")
        .unwrap()["xml"]
        .as_str()
        .unwrap();
    for tag in ["visual", "collision"] {
        let xml = base.replace("</link>", &format!("<{tag}/></link>"));
        let error = load_urdf_str(&xml, "missing-geometry.urdf").unwrap_err();
        assert_eq!(error.code, ErrorCode::Missing);
        assert_eq!(error.source.uri, "missing-geometry.urdf");
        assert_eq!(error.source.line, 1);
        assert_eq!(
            error.source.column as usize,
            xml.find(&format!("<{tag}")).unwrap() + 1
        );
    }
}
#[test]
fn dependency_errors_preserve_original_root_location_and_context() {
    let xml = ORIGINAL.replace("name=\"test\"", "name=\"test&unbound;\"");
    let error = load_urdf_str(&xml, "dependency.urdf").unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!(error.source.uri, "dependency.urdf");
    assert_eq!((error.source.line, error.source.column), (1, 1));
    assert!(error.message.starts_with("URDF parsing failed:"));
}
