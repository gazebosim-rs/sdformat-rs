// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Supplemental graph projections; originals are in world_merge_originals.rs.
use sdformat::{ErrorCode, Model, Plugin, Resource, ResourceResolver, Sdf, Transform, load_str};
use serde_json::{Value, json};

struct Resolver<'a>(&'a Value);
impl ResourceResolver for Resolver<'_> {
    fn resolve(&self, uri: &str, _parent: &str) -> Result<Resource, String> {
        let key = match uri {
            "@RESOURCE@" => "resource",
            "@CHILD@" => "child",
            _ => return Err(uri.into()),
        };
        Ok(Resource {
            canonical_uri: uri.into(),
            xml: self.0[key].as_str().unwrap().into(),
        })
    }
}
fn corpus() -> Value {
    serde_json::from_str(include_str!("world_merge_cases.json")).unwrap()
}
fn pose(actual: Transform, expected: &Value) {
    let values = expected.as_array().unwrap();
    assert_eq!(values.len(), 7);
    for (a, e) in actual
        .translation
        .into_iter()
        .chain(actual.rotation)
        .zip(values)
    {
        assert!((a - e.as_f64().unwrap()).abs() <= 1e-12, "{a} != {e}");
    }
}
fn plugins(actual: &[Plugin], native: &Value) {
    assert_eq!(
        json!(
            actual
                .iter()
                .map(|p| json!({"name":p.name,"filename":p.filename}))
                .collect::<Vec<_>>()
        ),
        *native
    );
}
fn source_uri(actual: &str, native: &Value) {
    // LoadSdfString leaves direct native Element.FilePath empty; Rust keeps the
    // caller-supplied Source. Imported file identities are compared directly.
    let source = native.as_str().unwrap();
    assert_eq!(actual, if source.is_empty() { "@INPUT@" } else { source });
}
fn model(sdf: &Sdf, actual: &Model, native: &Value, parent: &str) {
    assert_eq!(actual.name, native["name"]);
    assert_eq!(json!(actual.raw_namespace), native["raw"]);
    assert_eq!(json!(actual.namespace()), native["resolved"]);
    assert_eq!(actual.is_static, native["static"]);
    assert_eq!(actual.canonical_link, native["canonical"]);
    assert_eq!(actual.placement_frame, native["placement"]);
    assert_eq!(actual.pose.relative_to, native["relative"]);
    pose(actual.pose.transform, &native["pose"]);
    assert_eq!(native["pose_errors"], json!([]));
    pose(
        sdf.pose_of(&actual.scoped_name, parent).unwrap(),
        &native["resolved_pose"],
    );
    assert_eq!(
        actual.links.len(),
        native["links"].as_u64().unwrap() as usize
    );
    assert_eq!(
        json!(actual.frames.iter().map(|f| &f.name).collect::<Vec<_>>()),
        native["frames"]
    );
    source_uri(&actual.source.uri, &native["element_file"]);
    if let Some(include) = &actual.include {
        assert_eq!(include.uri, native["uri"]);
    }
    plugins(&actual.plugins, &native["plugins"]);
    assert_eq!(
        actual.models.len(),
        native["models"].as_array().unwrap().len()
    );
    for (a, n) in actual
        .models
        .iter()
        .zip(native["models"].as_array().unwrap())
    {
        model(sdf, a, n, &actual.scoped_name);
    }
    for (name, n) in actual
        .links
        .iter()
        .map(|l| &l.name)
        .zip(native["links_data"].as_array().unwrap())
        .chain(
            actual
                .frames
                .iter()
                .map(|f| &f.name)
                .zip(native["frames_data"].as_array().unwrap()),
        )
    {
        assert_eq!(name, n["name"].as_str().unwrap());
        for target in ["default", "model"] {
            assert_eq!(n["semantic"][target]["errors"], json!([]));
            pose(
                sdf.pose_of(
                    &format!("{}::{name}", actual.scoped_name),
                    &actual.scoped_name,
                )
                .unwrap(),
                &n["semantic"][target]["pose"],
            );
        }
        // Native model-local lookup does not expose these root aliases.
        for target in ["root", "absolute_root"] {
            assert_eq!(n["semantic"][target]["errors"], json!([26]));
            assert!(n["semantic"][target]["pose"].is_null());
        }
    }
}
fn scoped(world: &str, name: &str) -> String {
    if name.is_empty() || name == "world" {
        world.into()
    } else {
        format!("{world}::{name}")
    }
}
fn semantic(sdf: &Sdf, world: &str, frame: &str, observations: &Value, default_target: &str) {
    for row in observations.as_array().unwrap() {
        if row["resolved"]["errors"] == json!([]) {
            let target = row["target"].as_str().unwrap();
            let target = if target.is_empty() {
                default_target.into()
            } else {
                scoped(world, target)
            };
            pose(
                sdf.pose_of(frame, &target).unwrap(),
                &row["resolved"]["pose"],
            );
        } else {
            // Error vectors/null remain native observations; Rust's global API
            // does not promise the same model-local special-name lookup API.
            assert!(row["resolved"]["pose"].is_null());
            assert!(!row["resolved"]["errors"].as_array().unwrap().is_empty());
        }
    }
}
#[test]
fn all_native_world_merge_graphs_and_specific_rejections() {
    let cases = corpus();
    assert_eq!(cases.as_array().unwrap().len(), 16);
    let mut counts = [0, 0];
    for row in cases.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let result = load_str(row["xml"].as_str().unwrap(), "@INPUT@", &Resolver(row));
        let native = &row["native"];
        if native["errors"] != json!([]) {
            let error = result.expect_err(id);
            let expected = match id {
                "merge-direct-link-invalid" | "merge-direct-link-and-nested-invalid" => {
                    ErrorCode::MergeIncludeUnsupported
                }
                "merge-parent-name-collision" => ErrorCode::DuplicateName,
                "merge-unknown-parent-frame" | "merge-unknown-placement" => {
                    ErrorCode::UnknownReference
                }
                _ => panic!("unclassified native error {id}"),
            };
            assert_eq!(error.code, expected, "{id}: {error}");
            counts[1] += 1;
            continue;
        }
        let sdf = result.unwrap_or_else(|e| panic!("{id}: {e}"));
        sdf.validate_non_frame_poses().unwrap();
        assert_eq!(sdf.worlds.len(), 1);
        assert!(sdf.models.is_empty());
        let world = &sdf.worlds[0];
        let n = &native["worlds"][0];
        assert_eq!(world.name, n["name"]);
        source_uri(&world.source.uri, &n["element_file"]);
        assert_eq!(world.models.len(), n["models"].as_array().unwrap().len());
        for (a, n) in world.models.iter().zip(n["models"].as_array().unwrap()) {
            model(&sdf, a, n, &world.name);
        }
        assert_eq!(
            world.frames.len(),
            n["frames_data"].as_array().unwrap().len()
        );
        for (a, n) in world
            .frames
            .iter()
            .zip(n["frames_data"].as_array().unwrap())
        {
            assert_eq!(a.name, n["name"]);
            source_uri(&a.source.uri, &n["element_file"]);
            assert_eq!(a.attached_to, n["attached_to"]);
            assert_eq!(a.pose.relative_to, n["relative"]);
            pose(a.pose.transform, &n["raw_pose"]);
            let name = scoped(&world.name, &a.name);
            assert_eq!(n["attached_body"]["errors"], json!([]));
            assert_eq!(
                sdf.attached_body(&name).unwrap(),
                scoped(&world.name, n["attached_body"]["body"].as_str().unwrap())
            );
            semantic(&sdf, &world.name, &name, &n["semantic"], &world.name);
        }
        assert_eq!(
            world.joints.len(),
            n["joints_data"].as_array().unwrap().len()
        );
        for (a, n) in world
            .joints
            .iter()
            .zip(n["joints_data"].as_array().unwrap())
        {
            assert_eq!(a.name, n["name"]);
            source_uri(&a.source.uri, &n["element_file"]);
            assert_eq!(a.parent, n["parent"]);
            assert_eq!(a.child, n["child"]);
            assert_eq!(a.pose.relative_to, n["relative"]);
            assert_eq!(a.kind, sdformat::JointKind::Fixed);
            assert_eq!(n["type"], 3);
            pose(a.pose.transform, &n["raw_pose"]);
            for (reference, key) in [(&a.parent, "parent_body"), (&a.child, "child_body")] {
                assert_eq!(n[key]["errors"], json!([]));
                let endpoint = sdf.resolve_name(&world.name, reference).unwrap();
                assert_eq!(
                    sdf.attached_body(&endpoint).unwrap(),
                    scoped(&world.name, n[key]["body"].as_str().unwrap())
                );
            }
            semantic(
                &sdf,
                &world.name,
                &scoped(&world.name, &a.name),
                &n["semantic"],
                &sdf.resolve_name(&world.name, &a.child).unwrap(),
            );
        }
        plugins(&world.plugins, &n["plugins"]);
        let merged = id != "merge-false-container" && id != "direct-global-joint-control";
        assert_eq!(world.merged_includes.len(), usize::from(merged));
        if let Some(record) = world.merged_includes.first() {
            assert_eq!(record.include.uri, "@RESOURCE@");
            assert_eq!(record.model_source.uri, "@RESOURCE@");
            let xml = row["xml"].as_str().unwrap();
            let start = xml.find("<include").unwrap();
            let end = xml[start..].find("</include>").unwrap() + start + "</include>".len();
            assert_eq!(record.include.xml, xml[start..end]);
            assert_eq!(record.include.source.uri, "@INPUT@");
            assert!(world.frames.iter().any(|f| f.name == record.proxy_frame));
        }
        counts[0] += 1;
    }
    assert_eq!(counts, [11, 5]);
}

#[test]
fn world_joint_endpoint_axis_and_deferred_sensor_validation() {
    let base = "<sdf version='1.12'><world name='w'><model name='a'><link name='l'/></model><model name='b'><link name='l'/></model>@JOINT@</world></sdf>";
    let joint =
        "<joint name='j' type='revolute'><parent>a::l</parent><child>b::l</child>@AXIS@</joint>";
    let xml = base.replace(
        "@JOINT@",
        &joint.replace(
            "@AXIS@",
            "<axis><xyz expressed_in='missing'>0 0 1</xyz></axis>",
        ),
    );
    let error = load_str(&xml, "axis.sdf", &sdformat::NoResources).unwrap_err();
    assert_eq!(error.code, ErrorCode::JointAxisExpressedInInvalid);
    assert!(error.message.contains("world with name[w]"));
    for child in ["a::l", "world"] {
        let xml = base.replace(
            "@JOINT@",
            &joint.replace("@AXIS@", "").replace("b::l", child),
        );
        assert_eq!(
            load_str(&xml, "endpoints.sdf", &sdformat::NoResources)
                .unwrap_err()
                .code,
            ErrorCode::InvalidValue
        );
    }
    let joint = joint.replace("<parent>a::l</parent>", "<parent>world</parent>");
    let xml = base.replace("@JOINT@", &joint.replace("@AXIS@", "<axis><xyz expressed_in='a::l'>0 0 1</xyz></axis><sensor name='s' type='imu'><pose relative_to='missing'>0 0 0 0 0 0</pose></sensor>"));
    let sdf = load_str(&xml, "sensor.sdf", &sdformat::NoResources).unwrap();
    assert_eq!(
        sdf.validate_non_frame_poses().unwrap_err().code,
        ErrorCode::UnknownReference
    );
    let valid = xml.replace("relative_to='missing'", "relative_to='a::l'");
    let sdf = load_str(&valid, "sensor.sdf", &sdformat::NoResources).unwrap();
    sdf.validate_non_frame_poses().unwrap();
    assert_eq!(
        sdf.resolve_axis(sdf.worlds[0].joints[0].axis.as_ref().unwrap(), "w::j", "w")
            .unwrap(),
        [0., 0., 1.]
    );
}
