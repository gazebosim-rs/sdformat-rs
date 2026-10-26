// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
// Native Root.LoadSdfString accepts both exact supplemental payloads.
use sdformat::{NoResources, load_str};

#[test]
fn unknown_mimic_content_keeps_the_known_payload() {
    let base = include_str!("fixtures/joint_axis_mimic.sdf");
    for xml in [
        base.replace(
            "<reference>3</reference>",
            "<reference>3</reference><bogus/>",
        ),
        base.replace(
            "<mimic joint='source_joint'>",
            "<mimic joint='source_joint' extra='x'>",
        ),
    ] {
        let sdf = load_str(&xml, "joint_axis_mimic_extensions.sdf", &NoResources)
            .expect("native accepts the supplemental mimic content");
        let mimic = sdf.models[0]
            .joint_by_name("follow_joint")
            .unwrap()
            .axis(0)
            .unwrap()
            .mimic
            .as_ref()
            .unwrap();
        assert_eq!(mimic.joint, "source_joint");
        assert_eq!(mimic.axis, "axis");
        assert_eq!(
            (mimic.multiplier, mimic.offset, mimic.reference),
            (4., 2., 3.)
        );
    }
}
