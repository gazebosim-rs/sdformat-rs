// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use super::*;

#[test]
fn disconnected_scope_is_a_typed_error_not_an_ancestry_panic() {
    let sdf = crate::load_str("<sdf version='1.12'><world name='w'><model name='m'><link name='a'/></model><model name='other'><link name='b'/></model></world></sdf>", "scope.sdf", &crate::NoResources).unwrap();
    let source = &sdf.worlds[0].models[0].source;
    let error = sdf
        .graphs
        .joint_scope_pose("w::m::a", "w::other", true, source)
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidValue);
    assert_eq!(&error.source, source);
}
