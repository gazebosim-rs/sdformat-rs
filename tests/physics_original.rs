/*
 * Copyright (C) 2018 Open Source Robotics Foundation
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
use sdformat::Physics;

#[test]
fn original_physics_default_construction() {
    let mut physics = Physics::default();
    assert!(physics.xml.is_none());
    assert!(physics.name.is_empty());
    physics.name = "test_physics".into();
    assert_eq!(physics.name, "test_physics");
    assert!(!physics.is_default);
    physics.is_default = true;
    assert!(physics.is_default);
    assert_eq!(physics.engine_type, "ode");
    physics.engine_type = "bullet".into();
    assert_eq!(physics.engine_type, "bullet");
    assert_eq!(physics.max_step_size, 0.001);
    physics.max_step_size = 1.234;
    assert_eq!(physics.max_step_size, 1.234);
    assert_eq!(physics.real_time_factor, 1.0);
    physics.real_time_factor = 2.45;
    assert_eq!(physics.real_time_factor, 2.45);
}
