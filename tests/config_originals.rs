/*
 * Copyright 2017 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

use sdformat::select_model_config;

// Selection-only portions of ModelVersionsTest. Filesystem path assembly remains
// outside this pure API; these cases are partial, not retired original tests.
#[test]
fn no_version_tag_filename_selection() {
    let selected = select_model_config(
        include_str!("fixtures/config_no_version.config"),
        "model.config",
    )
    .unwrap();
    assert_eq!(selected.file_name, "model-1_2.sdf");
}

#[test]
fn correct_version_filename_selection() {
    let selected = select_model_config(
        include_str!("fixtures/config_versions.config"),
        "model.config",
    )
    .unwrap();
    assert_eq!(selected.file_name, "model-1_4.sdf");
}
