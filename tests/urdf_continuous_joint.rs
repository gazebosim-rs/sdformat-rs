/*
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
 *
 */

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{ErrorCode, Geometry, Model, Sdf, Transform, load_urdf_str};
use serde_json::Value;
const ORIGINAL: &str = include_str!("fixtures/urdf_continuous_original.urdf");
fn data() -> Value {
    serde_json::from_str(include_str!("urdf_continuous_joint.json")).unwrap()
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
    result += &format!(
        "CANONICAL {}\nORDER{}\n",
        hex(&model.canonical_link),
        model
            .loaded_element
            .as_ref()
            .unwrap()
            .children()
            .iter()
            .map(|v| format!(" {}", hex(v.tag())))
            .collect::<String>()
    );
    for link in &model.links {
        result += &format!(
            "LINK {}{} {} {}\nINERTIAL{}{}\nLINKFRAME {}\nRESOLVED link 0{}\nRESOLVED inertial 0{}\n",
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
            hex(&link.pose.relative_to),
            pose(&sdf.pose_of(&link.scoped_name, &model.scoped_name).unwrap()),
            pose(
                &sdf.resolve_pose(
                    &link.inertial.pose,
                    &model.scoped_name,
                    &link.scoped_name,
                    &model.scoped_name
                )
                .unwrap()
            )
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
    for joint in &model.joints {
        assert_eq!(joint.kind, sdformat::JointKind::Revolute);
        let scoped = format!("{}::{}", model.scoped_name, joint.name);
        result += &format!(
            "JOINT {} 6 {} {} {}{}\nRESOLVED joint 0{}\n",
            hex(&joint.name),
            hex(&joint.parent),
            hex(&joint.child),
            hex(&joint.pose.relative_to),
            pose(&joint.pose.transform),
            pose(&sdf.joint_pose(&scoped, "__model__").unwrap())
        );
        for endpoint in [&joint.parent, &joint.child] {
            let name = sdf.resolve_name(&model.scoped_name, endpoint).unwrap();
            assert_eq!(sdf.attached_body(&name).unwrap(), name);
        }
        result += &format!(
            "ENDPOINTS 0 0 {} {}\n",
            hex(&joint.parent),
            hex(&joint.child)
        );
        let axis = joint.axis.as_ref().unwrap();
        result += &format!(
            "AXIS{} {}\nRESOLVED axis 0{}\n",
            numbers(axis.xyz.into_iter().chain([
                axis.lower,
                axis.upper,
                axis.effort,
                axis.velocity,
                axis.dynamics.damping,
                axis.dynamics.friction,
                axis.dynamics.spring_reference,
                axis.dynamics.spring_stiffness,
                axis.stiffness,
                axis.dissipation
            ])),
            hex(&axis.expressed_in),
            numbers(sdf.resolve_axis(axis, &scoped, &model.scoped_name).unwrap())
        );
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
    assert_eq!(
        actual.len(),
        expected.len(),
        "{name}: actual{actual:?} expected{expected:?}"
    );
    for (actual, expected) in actual.iter().zip(&expected) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "{name}: {actual:?} {expected:?}"
        );
        let pose_start = match actual[0] {
            "MODEL" => Some(4),
            "LINK" | "VISUAL" | "COLLISION" => Some(2),
            "INERTIAL" => Some(8),
            "JOINT" => Some(6),
            "COPY" => Some(3),
            "RESOLVED" if actual[1] != "axis" => Some(3),
            _ => None,
        };
        for i in 0..actual.len() {
            let quaternion = pose_start.is_some_and(|s| (s + 3..s + 7).contains(&i));
            let resolved = actual[0] == "RESOLVED" && i >= 3;
            if quaternion {
                continue;
            }
            if resolved {
                let a = f64::from_bits(actual[i].parse().unwrap());
                let b = f64::from_bits(expected[i].parse().unwrap());
                assert!(
                    (a - b).abs() <= 1e-12,
                    "{name} resolved field{i}: {a:?} {b:?}"
                );
            } else {
                assert_eq!(actual[i], expected[i], "{name} field{i}");
            }
        }
        if let Some(start) = pose_start {
            let a: Vec<_> = actual[start + 3..start + 7]
                .iter()
                .map(|s| f64::from_bits(s.parse().unwrap()))
                .collect();
            let b: Vec<_> = expected[start + 3..start + 7]
                .iter()
                .map(|s| f64::from_bits(s.parse().unwrap()))
                .collect();
            assert!(
                a.iter().zip(&b).all(|(a, b)| (a - b).abs() <= 1e-12)
                    || a.iter().zip(&b).all(|(a, b)| (a + b).abs() <= 1e-12),
                "{name} quaternion {a:?} {b:?}"
            );
        }
    }
}
#[test]
fn native_two_link_joint_frames_axes_limits_dynamics_and_strict_boundaries() {
    for row in data()["cases"].as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let result = load_urdf_str(row["xml"].as_str().unwrap(), "joint-input.urdf");
        if row["policy"] == "admit" {
            let import = result.unwrap_or_else(|e| panic!("{name}: {e:?}"));
            assert!(import.sdf.worlds.is_empty());
            assert_eq!(import.sdf.models.len(), 1);
            let model = &import.sdf.models[0];
            assert_eq!((model.links.len(), model.joints.len()), (2, 1));
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
        } else {
            let error = result.expect_err(name);
            let code = match row["rust_policy"].as_str().unwrap() {
                "InvalidValue" => ErrorCode::InvalidValue,
                "Missing" => ErrorCode::Missing,
                "Unsupported" => ErrorCode::Unsupported,
                "UnknownReference" => ErrorCode::UnknownReference,
                "Version" => ErrorCode::Version,
                _ => panic!("unrecognized policy"),
            };
            assert_eq!(error.code, code, "{name}: {error:?}");
            assert_eq!(error.source.uri, "joint-input.urdf");
        }
    }
}
#[test]
fn original_valid_continuous_jointed_urdf_preserves_all_five_original_observations() {
    let import = load_urdf_str(ORIGINAL, "continuous-original.urdf").unwrap();
    let model = import.sdf.models.first().unwrap();
    assert!(model.links.iter().any(|v| v.name == "link1"));
    assert!(model.links.iter().any(|v| v.name == "link2"));
    assert!(model.joints.iter().any(|v| v.name == "joint1_2"));
    assert_eq!((model.links.len(), model.joints.len()), (2, 1));
    let joint = &model.joints[0];
    assert_eq!(joint.pose.relative_to, "link1");
    assert_eq!(model.links[1].pose.relative_to, "joint1_2");
    let axis = joint.axis.as_ref().unwrap();
    assert_eq!(axis.xyz, [1., 0., 0.]);
    assert_eq!(
        (axis.lower, axis.upper, axis.effort, axis.velocity),
        (
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::INFINITY,
            f64::INFINITY
        )
    );
}
#[test]
fn all_accepted_single_link_generated_xml_bytes_remain_identical() {
    let corpus: Value = serde_json::from_str(include_str!("urdf_single_link.json")).unwrap();
    let expected: Value =
        serde_json::from_str(include_str!("urdf_single_link_converted.json")).unwrap();
    let mut count = 0;
    for row in corpus["cases"].as_array().unwrap() {
        if row["rust_policy"] == "accepted" {
            let name = row["name"].as_str().unwrap();
            let import = load_urdf_str(row["xml"].as_str().unwrap(), "input.urdf").unwrap();
            assert_eq!(
                import.converted_xml(),
                expected[name].as_str().unwrap(),
                "{name}"
            );
            count += 1;
        }
    }
    assert_eq!(count, expected.as_object().unwrap().len());
    assert_eq!(count, 31);
}
#[test]
fn new_joint_nodes_reject_unknown_attributes_and_children_before_backend_parsing() {
    let xml = ORIGINAL.replace(
        "</joint>",
        "<axis xyz='0 0 1'/><limit effort='3' velocity='4'/><dynamics damping='0.2'/></joint>",
    );
    for tag in ["joint", "parent", "child", "axis", "limit", "dynamics"] {
        let needle = format!("<{tag}");
        let at = xml.find(&needle).unwrap() + needle.len();
        let mut changed = xml.clone();
        changed.insert_str(at, " unknown='1'");
        let e = load_urdf_str(&changed, "unknown-joint.urdf").unwrap_err();
        assert_eq!(e.code, ErrorCode::Unsupported, "{tag}: {e:?}");
        assert_eq!(e.source.uri, "unknown-joint.urdf");
        let end = xml[at..].find('>').unwrap() + at;
        let mut changed = xml.clone();
        if changed.as_bytes()[end - 1] == b'/' {
            changed.replace_range(end - 1..=end, &format!("><unknown/></{tag}>"));
        } else {
            changed.insert_str(end + 1, "<unknown/>");
        }
        assert_eq!(
            load_urdf_str(&changed, "unknown-child.urdf")
                .unwrap_err()
                .code,
            ErrorCode::Unsupported,
            "{tag}"
        );
    }
}
#[test]
fn original_and_generated_joint_sources_and_snapshots_remain_distinct_and_immutable() {
    let mut import = load_urdf_str(ORIGINAL, "owned-joint.urdf").unwrap();
    assert_eq!(import.original_xml(), ORIGINAL);
    assert_eq!(import.original_source().uri, "owned-joint.urdf");
    let generated = import.converted_xml().to_owned();
    let source = import.converted_source().clone();
    assert_eq!(source.uri, "generated:urdf-to-sdf:owned-joint.urdf");
    let model = &mut import.sdf.models[0];
    let snapshot = model.loaded_element.clone().unwrap();
    let joint = &mut model.joints[0];
    assert_eq!(joint.source.uri, source.uri);
    assert_eq!(
        joint.source.column as usize,
        generated.find("<joint").unwrap() + 1
    );
    assert_eq!(joint.source.line, 1);
    let mut copied = joint.clone();
    let mut assigned = copied.clone();
    assigned.clone_from(&copied);
    assert_eq!(assigned.parent, joint.parent);
    copied.name = "changed".into();
    copied.parent = "changed_parent".into();
    copied
        .axis
        .as_mut()
        .unwrap()
        .try_set_xyz([0., 1., 0.])
        .unwrap();
    assert_eq!(joint.name, "joint1_2");
    assert_eq!(joint.parent, "link1");
    assert_eq!(assigned.axis.as_ref().unwrap().xyz, [1., 0., 0.]);
    joint.name = "typed-edit".into();
    assert_eq!(model.loaded_element.as_ref(), Some(&snapshot));
    assert_eq!(import.original_xml(), ORIGINAL);
    assert_eq!(import.converted_xml(), generated);
    assert_eq!(import.converted_source(), &source);
}

#[test]
fn native_joint_copy_raw_values_remain_owned_without_graph_or_element_mutation_claims() {
    let import = load_urdf_str(ORIGINAL, "copy.urdf").unwrap();
    let original = &import.sdf.models[0].joints[0];
    let mut copied = original.clone();
    let mut assigned = original.clone();
    assigned.clone_from(&copied);
    copied.name = "changed".into();
    copied.parent = "changed_parent".into();
    copied.pose.transform = Transform {
        translation: [4., 5., 6.],
        rotation: glam::DQuat::from_euler(glam::EulerRot::ZYX, 0.3, 0.2, 0.1).to_array(),
    };
    copied
        .axis
        .as_mut()
        .unwrap()
        .try_set_xyz([0., 1., 0.])
        .unwrap();
    let mut actual = "ERRORS 0\n".to_owned();
    for value in [original, &assigned, &copied] {
        actual += &format!(
            "COPY {} {}{}{}\n",
            hex(&value.name),
            hex(&value.parent),
            pose(&value.pose.transform),
            numbers(value.axis.as_ref().unwrap().xyz)
        );
    }
    compare(
        &actual,
        data()["clone"]["stdout"].as_str().unwrap(),
        "native copy",
    );
}

#[test]
fn native_canonical_and_default_child_frame_resolution_match_existing_graphs() {
    for row in data()["frames"].as_array().unwrap() {
        let import = load_urdf_str(row["xml"].as_str().unwrap(), "frames.urdf").unwrap();
        let model = &import.sdf.models[0];
        let joint = &model.joints[0];
        let scoped = format!("{}::{}", model.scoped_name, joint.name);
        let mut lines = row["stdout"].as_str().unwrap().lines();
        assert_eq!(lines.next().unwrap(), "ERRORS 0");
        let canonical = lines.next().unwrap().strip_prefix("CANONICAL ").unwrap();
        assert_eq!(
            import.sdf.attached_body(&model.scoped_name).unwrap(),
            format!("{}::{canonical}", model.scoped_name)
        );
        for line in lines {
            let fields: Vec<_> = line.split_whitespace().collect();
            assert_eq!(fields[0], "TARGET");
            assert_eq!(&fields[2..4], ["0", "0"]);
            let target = if fields[1] == "<default>" {
                ""
            } else {
                fields[1]
            };
            let pose = import.sdf.joint_pose(&scoped, target).unwrap();
            let axis_target = match target {
                "" => scoped.clone(),
                "__model__" => model.scoped_name.clone(),
                name => import.sdf.resolve_name(&model.scoped_name, name).unwrap(),
            };
            let axis = import
                .sdf
                .resolve_axis(joint.axis.as_ref().unwrap(), &scoped, &axis_target)
                .unwrap();
            let actual: Vec<_> = pose
                .translation
                .into_iter()
                .chain(pose.rotation)
                .chain(axis)
                .collect();
            let expected: Vec<f64> = fields[4..]
                .iter()
                .map(|value| value.parse().unwrap())
                .collect();
            assert_eq!(actual.len(), expected.len());
            for i in [0, 1, 2, 7, 8, 9] {
                assert!(
                    (actual[i] - expected[i]).abs() <= 1e-12,
                    "{} target{target} field{i}",
                    row["name"]
                );
            }
            assert!(
                actual[3..7]
                    .iter()
                    .zip(&expected[3..7])
                    .all(|(a, b)| (a - b).abs() <= 1e-12)
                    || actual[3..7]
                        .iter()
                        .zip(&expected[3..7])
                        .all(|(a, b)| (a + b).abs() <= 1e-12)
            );
        }
    }
}
