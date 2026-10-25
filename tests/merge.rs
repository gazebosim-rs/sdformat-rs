// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;
struct Resolver<'a>(&'a serde_json::Value);
impl ResourceResolver for Resolver<'_> {
    fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
        let key = match uri {
            "model://child" => "resource",
            "model://grandchild" => "grandchild",
            _ => return Err(uri.into()),
        };
        Ok(Resource {
            canonical_uri: format!(
                "cached://{}/model.sdf",
                uri.strip_prefix("model://").unwrap()
            ),
            xml: self.0[key].as_str().ok_or("missing test resource")?.into(),
        })
    }
}
#[test]
fn native_model_merge_acceptance() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("merge.json")).unwrap();
    for row in cases.as_array().unwrap() {
        let result = load_str(row["xml"].as_str().unwrap(), "root.sdf", &Resolver(row));
        if row.get("gap").is_some() {
            assert!(row["accepted"].as_bool().unwrap());
            assert_eq!(result.unwrap_err().code, ErrorCode::Unsupported);
            continue;
        }
        assert_eq!(
            result.is_ok(),
            row["accepted"].as_bool().unwrap(),
            "{}: {result:?}",
            row["id"]
        );
    }
}

fn pose(value: Transform) -> Vec<f64> {
    value
        .translation
        .into_iter()
        .chain(value.rotation)
        .collect()
}
fn resolved(result: Result<Transform, Diagnostic>) -> serde_json::Value {
    match result {
        Ok(value) => serde_json::json!({"errors":0,"pose":pose(value)}),
        Err(_) => serde_json::json!({"errors":1,"pose":null}),
    }
}
fn describe(sdf: &Sdf, model: &Model, parent: &str) -> serde_json::Value {
    let links: Vec<_> = model.links.iter().map(|link| {
        let collisions:Vec<_>=link.collisions.iter().map(|c|serde_json::json!({
            "name":c.name,"relative_to":c.pose.relative_to,"raw_pose":pose(c.pose.transform),
            "resolved":resolved(sdf.resolve_pose(&c.pose,&model.scoped_name,&link.scoped_name,&link.scoped_name))
        })).collect();
        serde_json::json!({"name":link.name,"relative_to":link.pose.relative_to,"raw_pose":pose(link.pose.transform),
            "resolved":resolved(sdf.pose_of(&link.scoped_name,&model.scoped_name)),"collisions":collisions})
    }).collect();
    let frames: Vec<_> = model.frames.iter().map(|frame| {
        let name=format!("{}::{}",model.scoped_name,frame.name);
        let (body_errors,body)=match sdf.attached_body(&name) {
            Ok(body)=>(0,body.strip_prefix(&format!("{}::",model.scoped_name)).unwrap_or(body).to_string()),
            Err(_)=>(1,String::new()),
        };
        serde_json::json!({"name":frame.name,"attached_to":frame.attached_to,"relative_to":frame.pose.relative_to,
            "raw_pose":pose(frame.pose.transform),"resolved":resolved(sdf.pose_of(&name,&model.scoped_name)),"body_errors":body_errors,"body":body})
    }).collect();
    let joints: Vec<_> = model.joints.iter().map(|joint| {
        let name=format!("{}::{}",model.scoped_name,joint.name);
        let child=sdf.resolve_name(&model.scoped_name,&joint.child).unwrap();
        let axes:Vec<_>=[&joint.axis,&joint.axis2].into_iter().map(|axis|axis.as_ref().map(|axis| {
            let xyz=sdf.resolve_axis(axis,&name,&name);
            serde_json::json!({"expressed_in":axis.expressed_in,"xyz":axis.xyz,"errors":u8::from(xyz.is_err()),"resolved":xyz.ok()})
        })).collect();
        serde_json::json!({"name":joint.name,"parent":joint.parent,"child":joint.child,"relative_to":joint.pose.relative_to,
            "raw_pose":pose(joint.pose.transform),"resolved":resolved(sdf.pose_of(&name,&child)),"axes":axes})
    }).collect();
    let models: Vec<_> = model
        .models
        .iter()
        .map(|nested| describe(sdf, nested, &model.scoped_name))
        .collect();
    serde_json::json!({"name":model.name,"is_static":model.is_static,"canonical":model.canonical_link_and_relative_name().1,
        "relative_to":model.pose.relative_to,"raw_pose":pose(model.pose.transform),"resolved":resolved(sdf.pose_of(&model.scoped_name,parent)),
        "links":links,"frames":frames,"joints":joints,"models":models,"plugins":model.plugins.iter().map(|p|serde_json::json!({"name":p.name,"filename":p.filename})).collect::<Vec<_>>()})
}
fn compare(actual: &serde_json::Value, expected: &serde_json::Value, path: &str) {
    use serde_json::Value;
    match expected {
        Value::Number(n) if n.is_f64() => assert!(
            (actual.as_f64().unwrap() - n.as_f64().unwrap()).abs() <= 1e-12,
            "{path}: {actual} != {expected}"
        ),
        Value::Array(values) => {
            let actual = actual.as_array().unwrap();
            assert_eq!(actual.len(), values.len(), "{path}");
            for (i, (a, e)) in actual.iter().zip(values).enumerate() {
                compare(a, e, &format!("{path}/{i}"));
            }
        }
        Value::Object(values) => {
            let actual = actual.as_object().unwrap();
            assert_eq!(actual.len(), values.len(), "{path}");
            for (key, value) in values {
                compare(&actual[key], value, &format!("{path}/{key}"));
            }
        }
        _ => assert_eq!(actual, expected, "{path}"),
    }
}
#[test]
fn native_merge_frames_poses_axes_and_canonical_names() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("merge.json")).unwrap();
    for row in cases
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["accepted"] == true && row.get("gap").is_none())
    {
        let sdf = load_str(row["xml"].as_str().unwrap(), "root.sdf", &Resolver(row)).unwrap();
        compare(
            &describe(&sdf, &sdf.models[0], "::"),
            &row["model"],
            row["id"].as_str().unwrap(),
        );
    }
}

#[test]
fn merged_include_provenance_is_owned_and_preserves_nested_sources() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("merge.json")).unwrap();
    let row = cases
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "nested_placement")
        .unwrap();
    let sdf = load_str(row["xml"].as_str().unwrap(), "root.sdf", &Resolver(row)).unwrap();
    assert!(sdf.models[0].include.is_none());
    let records = sdf.models[0].merged_includes.clone();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].proxy_frame, "_merged__child__model__");
    assert_eq!(records[0].model_source.uri, "cached://child/model.sdf");
    assert_eq!(records[0].include.source.uri, "root.sdf");
    assert_eq!(records[1].proxy_frame, "_merged__grandchild__model__");
    assert_eq!(records[1].model_source.uri, "cached://grandchild/model.sdf");
    assert_eq!(records[1].include.source.uri, "cached://child/model.sdf");
    for record in &records {
        let frame = sdf.models[0]
            .frames
            .iter()
            .find(|f| f.name == record.proxy_frame)
            .unwrap();
        assert_eq!(frame.source, record.include.source);
    }
    let xml = row["xml"].as_str().unwrap();
    let span = &xml[xml.find("<include").unwrap()..xml.find("</include>").unwrap() + 10];
    assert_eq!(records[0].include.xml, span);
    drop(sdf);
    assert_eq!(records[0].include.uri, "model://child");
    assert_eq!(records[1].include.placement_frame.as_deref(), Some("gl"));
    assert_eq!(records[1].include.uri, "model://grandchild");
}

#[test]
fn merged_nonframe_errors_stay_deferred_and_world_scope_stays_unsupported() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("merge.json")).unwrap();
    let row = cases
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "deferred_nonframe")
        .unwrap();
    let sdf = load_str(row["xml"].as_str().unwrap(), "root.sdf", &Resolver(row)).unwrap();
    let error = sdf.validate_non_frame_poses().unwrap_err();
    assert_eq!(error.code, ErrorCode::UnknownReference);
    // The existing arbitrary-pose query API reports its graph-root Source.
    // Parsed collision provenance remains the original imported document.
    assert_eq!(error.source.uri, "root.sdf");
    assert_eq!(
        sdf.models[0]
            .links
            .iter()
            .find(|l| l.name == "l")
            .unwrap()
            .collisions[0]
            .source
            .uri,
        "cached://child/model.sdf"
    );
    let row = cases
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "nested_only")
        .unwrap();
    let error=load_str("<sdf version='1.12'><world name='w'><include merge='true'><uri>model://child</uri></include></world></sdf>","world.sdf",&Resolver(row)).unwrap_err();
    assert_eq!(error.code, ErrorCode::Unsupported);
    assert!(error.message.contains("world merge"));
}
