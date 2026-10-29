// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Qualified native six-case observations, with owned query/mutation supplements.
use sdformat::*;
use std::sync::Arc;

fn parsed(body: &str) -> Sdf {
    load_str(
        &format!("<sdf version='1.12'>{body}</sdf>"),
        "query.sdf",
        &NoResources,
    )
    .unwrap()
}

#[test]
fn absent_and_false_children_have_distinct_presence() {
    let absent = parsed("<model name='m'><link name='l'/></model>");
    let explicit = parsed(
        "<model name='m'><static>false</static><enable_wind>false</enable_wind><link name='l'/></model>",
    );
    let a = &absent.models[0];
    let b = &explicit.models[0];
    assert!(!a.is_static && !a.enable_wind && !b.is_static && !b.enable_wind);
    assert_eq!(
        a.element()
            .unwrap()
            .children()
            .iter()
            .map(ModelElementChild::tag)
            .collect::<Vec<_>>(),
        ["link"]
    );
    assert_eq!(
        b.element()
            .unwrap()
            .children()
            .iter()
            .map(ModelElementChild::tag)
            .collect::<Vec<_>>(),
        ["static", "enable_wind", "link"]
    );
    for tag in ["static", "enable_wind"] {
        assert!(!a.element().unwrap().has_child(tag));
        assert!(b.element().unwrap().has_child(tag));
    }
    assert!(
        b.element()
            .unwrap()
            .find_child("static")
            .unwrap()
            .name_attribute()
            .is_none()
    );
    assert!(!b.element().unwrap().has_child("never-present"));
    assert!(!b.element().unwrap().has_child(""));
}

#[test]
fn first_match_keeps_repeated_order_and_explicit_attribute() {
    let sdf = parsed(
        "<model name='m'><link name='l'/><plugin name='first' filename='a'/><plugin name='second' filename='b'/><custom:foo name='attribute-first'><name>child-name</name></custom:foo><custom:foo name='attribute-second'/></model>",
    );
    let e = sdf.models[0].element().unwrap();
    assert_eq!(
        e.children()
            .iter()
            .map(ModelElementChild::tag)
            .collect::<Vec<_>>(),
        ["link", "plugin", "plugin", "custom:foo", "custom:foo"]
    );
    assert_eq!(
        e.find_child("plugin").unwrap().name_attribute(),
        Some("first")
    );
    assert_eq!(
        e.find_child("custom:foo").unwrap().name_attribute(),
        Some("attribute-first")
    );
    assert!(e.find_child("PLUGIN").is_none());
}

#[test]
fn clone_shares_snapshot_and_typed_edits_leave_it_unchanged() {
    let retained;
    let copied;
    {
        let sdf = parsed(
            "<model name='m'><static>false</static><enable_wind>false</enable_wind><link name='l'/></model>",
        );
        let original = &sdf.models[0];
        let before = original.element().unwrap().clone();
        let mut edited = original.clone();
        assert!(std::ptr::eq(
            original.element().unwrap(),
            edited.element().unwrap()
        ));
        edited.name = "edited-name".into();
        edited.is_static = true;
        edited.enable_wind = true;
        assert_eq!(edited.element().unwrap(), &before);
        assert_eq!(original.name, "m");
        assert!(!original.is_static && !original.enable_wind);
        retained = Arc::clone(original.loaded_element.as_ref().unwrap());
        copied = edited;
    }
    // Native replay dropped Root while an edited Model remained alive too.
    assert_eq!(retained.as_ref(), copied.element().unwrap());
    assert_eq!(copied.name, "edited-name");
    assert!(retained.has_child("static"));
}

struct Included(&'static str);
impl ResourceResolver for Included {
    fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
        assert_eq!(uri, "probe-resource");
        Ok(Resource {
            canonical_uri: "resolved://model.sdf".into(),
            xml: self.0.into(),
        })
    }
}

#[test]
fn ordinary_include_preserves_loaded_children_and_override_metadata() {
    let resolver = Included(
        "<sdf version='1.12'><model name='inner'><link name='l'/><static>false</static><enable_wind>false</enable_wind><plugin name='first' filename='a'/><custom:foo name='source'/></model></sdf>",
    );
    let sdf = load_str("<sdf version='1.12'><include><uri>probe-resource</uri><name>renamed</name><static>true</static></include></sdf>", "include.sdf", &resolver).unwrap();
    let m = &sdf.models[0];
    assert_eq!(m.name, "renamed");
    assert!(m.is_static);
    assert!(!m.enable_wind);
    let e = m.element().unwrap();
    assert_eq!(
        e.children()
            .iter()
            .map(ModelElementChild::tag)
            .collect::<Vec<_>>(),
        ["link", "static", "enable_wind", "plugin", "custom:foo"]
    );
    assert_eq!(
        e.find_child("plugin").unwrap().name_attribute(),
        Some("first")
    );
    assert_eq!(
        e.find_child("custom:foo").unwrap().name_attribute(),
        Some("source")
    );
}

#[test]
fn include_generated_pose_static_and_plugins_append_once() {
    // Pinned parser.cc1839-1881 creates missing pose/static before appending plugins.
    let resolver =
        Included("<sdf version='1.12'><model name='inner'><link name='l'/></model></sdf>");
    let xml = "<sdf version='1.12'><include><uri>probe-resource</uri><pose>1 2 3 0 0 0</pose><static>true</static><plugin name='extra' filename='x'/></include></sdf>";
    let sdf = load_str(xml, "override.sdf", &resolver).unwrap();
    let e = sdf.models[0].element().unwrap();
    assert_eq!(
        e.children()
            .iter()
            .map(ModelElementChild::tag)
            .collect::<Vec<_>>(),
        ["link", "pose", "static", "plugin"]
    );
    assert_eq!(
        e.find_child("plugin").unwrap().name_attribute(),
        Some("extra")
    );
}

#[test]
fn merge_keeps_local_flags_and_filters_imported_unnamed_children() {
    let resolver = Included(
        "<sdf version='1.12'><model name='inner'><link name='l'/><enable_wind>true</enable_wind><static>false</static><unknown_element name='ignored'/><custom:foo name='one'/><custom:foo name='two'/><model name='nested'><link name='nested-link'/></model></model></sdf>",
    );
    let sdf = load_str("<sdf version='1.12'><model name='outer'><static>false</static><include merge='true'><uri>probe-resource</uri></include></model></sdf>", "merge.sdf", &resolver).unwrap();
    let m = &sdf.models[0];
    let e = m.element().unwrap();
    assert_eq!(
        e.children()
            .iter()
            .map(ModelElementChild::tag)
            .collect::<Vec<_>>(),
        [
            "static",
            "frame",
            "link",
            "custom:foo",
            "custom:foo",
            "model"
        ]
    );
    assert_eq!(
        e.find_child("frame").unwrap().name_attribute(),
        Some("_merged__inner__model__")
    );
    assert!(e.has_child("static"));
    assert!(!e.has_child("enable_wind"));
    assert!(!e.has_child("unknown_element"));
    assert_eq!(
        e.find_child("custom:foo").unwrap().name_attribute(),
        Some("one")
    );
    let nested = &m.models[0];
    assert_eq!(
        nested
            .element()
            .unwrap()
            .children()
            .iter()
            .map(ModelElementChild::tag)
            .collect::<Vec<_>>(),
        ["link", "pose"]
    );
}

#[test]
fn legacy_typed_rename_does_not_rewrite_loaded_name_attribute() {
    // Native Model.cc395 calls typed Link.SetName, which leaves Element unchanged.
    let sdf = load_str("<sdf version='1.6'><model name='m'><model name='same'><link name='inside'/></model><link name='same'/></model></sdf>", "legacy.sdf", &NoResources).unwrap();
    let m = &sdf.models[0];
    assert_eq!(m.links[0].name, "same_link");
    assert_eq!(
        m.element()
            .unwrap()
            .find_child("link")
            .unwrap()
            .name_attribute(),
        Some("same")
    );
}
