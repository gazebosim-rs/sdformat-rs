// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

#[test]
fn native_canonical_selection_is_independent_of_attachment() {
    let cases: serde_json::Value = serde_json::from_str(include_str!("canonical.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 17);
    for case in cases.as_array().unwrap() {
        let sdf = load_str(case["xml"].as_str().unwrap(), "canonical.sdf", &NoResources).unwrap();
        let model = &sdf.models[0];
        let (link, name) = model.canonical_link_and_relative_name();
        assert_eq!(name, case["name"].as_str().unwrap(), "{case}");
        assert_eq!(
            link.map(|link| link.name.as_str()),
            case["link"].as_str(),
            "{case}"
        );
        if case["id"] == "nested_static" {
            assert_eq!(
                sdf.attached_body(&model.scoped_name).unwrap(),
                "m::inner::nested"
            );
            assert_eq!(link.unwrap().scoped_name, "m::inner::nested");
        }
    }
}

#[test]
fn explicit_missing_scope_is_a_dom_lookup_miss() {
    let mut sdf = load_str(
        "<sdf version='1.12'><model name='m'><static>true</static><link name='l'/></model></sdf>",
        "canonical.sdf",
        &NoResources,
    )
    .unwrap();
    let model = &mut sdf.models[0];
    model.canonical_link = "absent::link".into();
    let (link, name) = model.canonical_link_and_relative_name();
    assert!(link.is_none());
    assert_eq!(name, "absent::link");
}
