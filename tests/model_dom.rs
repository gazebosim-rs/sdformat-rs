// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{Model, NoResources, load_str};

fn at<'a>(root: &'a Model, path: &str) -> &'a Model {
    path.split('/')
        .filter(|part| !part.is_empty())
        .fold(root, |model, index| {
            &model.models[index.parse::<usize>().unwrap()]
        })
}

#[test]
fn native_scoped_dom_lookups_return_the_stored_objects() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("model_dom.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let xml = match row["fixture"].as_str().unwrap() {
            "nested_model.sdf" => include_str!("fixtures/nested_model.sdf"),
            "double_pendulum.sdf" => include_str!("fixtures/double_pendulum.sdf"),
            _ => unreachable!(),
        };
        let sdf = load_str(xml, "lookup.sdf", &NoResources).unwrap();
        let root = &sdf.models[0];
        let scope = row["scope"].as_str().unwrap();
        let model = if scope.is_empty() {
            root
        } else {
            root.model_by_name(scope).unwrap()
        };
        let name = row["name"].as_str().unwrap();
        let path = row["path"].as_str().unwrap();
        if row["kind"] == "model" {
            let found = model.model_by_name(name);
            if path == "-" {
                assert!(found.is_none(), "{row}");
            } else {
                assert!(std::ptr::eq(found.unwrap(), at(root, path)), "{row}");
            }
        } else {
            let found = model.link_by_name(name);
            if path == "-" {
                assert!(found.is_none(), "{row}");
            } else {
                let (parent, index) = path.split_once('#').unwrap();
                assert!(
                    std::ptr::eq(
                        found.unwrap(),
                        &at(root, parent).links[index.parse::<usize>().unwrap()]
                    ),
                    "{row}"
                );
            }
        }
    }
}
