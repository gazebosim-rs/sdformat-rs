// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

struct LegacyResource;
impl ResourceResolver for LegacyResource {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String> {
        assert_eq!((uri, parent), ("old", "world-source"));
        Ok(Resource {
            canonical_uri:"asset:old/model.sdf".into(),
            xml:"<sdf version='1.6'>\n<model name='legacy'>\n<frame name='f'><pose>0 2 0 0 0 0</pose></frame>\n<link name='l'><pose frame='f'>1 0 0 0 0 0</pose></link>\n</model></sdf>".into(),
        })
    }
}
#[test]
fn included_conversion_uses_imported_version_and_preserves_source_positions() {
    let xml = "<sdf version='1.12'><model name='parent'><include><uri>old</uri><name>alias</name></include></model></sdf>";
    let sdf = load_str(xml, "world-source", &LegacyResource).unwrap();
    assert_eq!(sdf.version, "1.12");
    let imported = &sdf.models[0].models[0];
    let link = &imported.links[0];
    assert_eq!(imported.name, "alias");
    assert_eq!(link.pose.relative_to, "f");
    assert_eq!(link.source.uri, "asset:old/model.sdf");
    assert_eq!(link.source.line, 4);
    assert_eq!(link.source.column, 1);
    assert_eq!(
        sdf.pose_of(&link.scoped_name, &imported.scoped_name)
            .unwrap()
            .translation,
        [1., 2., 0.]
    );
    sdf.validate_non_frame_poses().unwrap();
}
#[test]
fn older_conversions_and_flattened_names_remain_explicit_boundaries() {
    for (xml, code) in [
        (
            "<sdf version='1.5'><model name='m'><link name='l'/></model></sdf>",
            ErrorCode::Version,
        ),
        (
            "<sdf version='1.6'><model name='m'><link name='nested::l'/></model></sdf>",
            ErrorCode::Unsupported,
        ),
        (
            "<sdf version='1.7'><model name='m'><link name='l'><pose frame=''/></link></model></sdf>",
            ErrorCode::Unsupported,
        ),
        (
            "<sdf version='1.7'><model name='m'><link name='a'/><link name='b'/><joint name='j' type='revolute'><parent>a</parent><child>b</child><axis><use_parent_model_frame>true</use_parent_model_frame></axis></joint></model></sdf>",
            ErrorCode::Unsupported,
        ),
    ] {
        assert_eq!(
            load_str(xml, "version-boundary", &NoResources)
                .unwrap_err()
                .code,
            code,
            "{xml}"
        );
    }
}

#[test]
fn previously_unsupported_shared_names_now_use_native_renaming() {
    let xml = "<sdf version='1.6'><model name='m'><link name='a'/><link name='b'/><joint name='a' type='fixed'><parent>a</parent><child>b</child></joint></model></sdf>";
    let sdf = load_str(xml, "legacy-name-boundary", &NoResources).unwrap();
    assert_eq!(sdf.models[0].joints[0].name, "a_joint");
    assert_eq!(sdf.models[0].links[0].name, "a");
}
