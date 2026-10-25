// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{Model, NoResources, load_str};

fn model_at<'a>(root: &'a Model, path: &str) -> &'a Model {
    path.split('/')
        .filter(|part| !part.is_empty())
        .fold(root, |model, index| {
            &model.models[index.parse::<usize>().unwrap()]
        })
}
fn member_at<'a, T>(
    root: &'a Model,
    path: &str,
    marker: char,
    members: fn(&Model) -> &[T],
) -> Option<&'a T> {
    if path == "-" {
        return None;
    }
    let (parent, index) = path.split_once(marker).unwrap();
    Some(&members(model_at(root, parent))[index.parse::<usize>().unwrap()])
}
fn same<T>(actual: Option<&T>, expected: Option<&T>, row: &serde_json::Value) {
    match (actual, expected) {
        (Some(a), Some(b)) => assert!(std::ptr::eq(a, b), "{row}"),
        (None, None) => (),
        _ => panic!("lookup presence mismatch: {row}"),
    }
}

#[test]
fn native_member_lookups_retain_scopes_and_actual_stored_identity() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("member_dom.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let xml = match row["fixture"].as_str().unwrap() {
            "nested_model.sdf" => include_str!("fixtures/nested_model.sdf"),
            "double_pendulum.sdf" => include_str!("fixtures/double_pendulum.sdf"),
            "model_multi_nested_model.sdf" => include_str!("fixtures/model_multi_nested_model.sdf"),
            _ => unreachable!(),
        };
        let mut sdf = load_str(xml, "lookup.sdf", &NoResources).unwrap();
        let mode = row["mode"].as_str().unwrap_or("normal");
        if mode != "normal" {
            let name = if mode == "missing_scope" {
                "missing::literal"
            } else {
                "mid_model::absent"
            };
            let root = &mut sdf.models[0];
            root.links[0].name = name.into();
            root.joints[0].name = name.into();
            root.frames[0].name = name.into();
        }
        let root = &sdf.models[0];
        let scope = row["scope"].as_str().unwrap();
        let model = if scope.is_empty() {
            root
        } else {
            root.model_by_name(scope).unwrap()
        };
        let name = row["name"].as_str().unwrap();
        let path = row["path"].as_str().unwrap();
        match row["kind"].as_str().unwrap() {
            "model" => same(
                model.model_by_name(name),
                if path == "-" {
                    None
                } else {
                    Some(model_at(root, path))
                },
                row,
            ),
            "link" => same(
                model.link_by_name(name),
                member_at(root, path, '#', |m| &m.links),
                row,
            ),
            "joint" => same(
                model.joint_by_name(name),
                member_at(root, path, '@', |m| &m.joints),
                row,
            ),
            "frame" => same(
                model.frame_by_name(name),
                member_at(root, path, '%', |m| &m.frames),
                row,
            ),
            _ => unreachable!(),
        }
    }
}
