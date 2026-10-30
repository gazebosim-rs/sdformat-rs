// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
//! One native Param text chunk; callers reject unrepresentable values first.
use quick_xml::escape::escape;
pub(crate) fn needs_cdata(value: &str) -> bool {
    value.contains("  ") || value.contains(['\t', '\n'])
}
pub(crate) fn text(value: &str) -> String {
    if needs_cdata(value) {
        format!("<![CDATA[{value}]]>")
    } else {
        escape(value).into_owned()
    }
}
