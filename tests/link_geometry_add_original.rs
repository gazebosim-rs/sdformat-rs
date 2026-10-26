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
// Observable values from DOMLink.AddCollision and DOMLink.AddVisual.
use sdformat::{Collision, Link, NoResources, Visual, load_str};

fn link_and_elements() -> (Link, Collision, Visual) {
    let mut link = load_str(
        "<sdf version='1.12'><model name='parent'><link name='link'><collision name='collision1'><geometry><sphere><radius>1</radius></sphere></geometry></collision><visual name='visual1'/></link></model></sdf>",
        "link-geometry-add.sdf",
        &NoResources,
    )
    .unwrap()
    .models
    .remove(0)
    .links
    .remove(0);
    let collision = link.collisions.remove(0);
    let visual = link.visuals.remove(0);
    (link, collision, visual)
}

#[test]
fn original_dom_link_add_collision() {
    let (mut link, collision, _) = link_and_elements();
    assert_eq!(link.collisions.len(), 0);
    assert!(link.add_collision_unique(collision));
    assert_eq!(link.collisions.len(), 1);
    let (_, duplicate, _) = link_and_elements();
    assert!(!link.add_collision_unique(duplicate));
    assert_eq!(link.collisions.len(), 1);

    link.clear_collisions();
    assert_eq!(link.collisions.len(), 0);
    let (_, collision, _) = link_and_elements();
    assert!(link.add_collision_unique(collision));
    assert_eq!(link.collisions.len(), 1);
    let collision_from_link = link.collisions.first().expect("CollisionByIndex(0)");
    assert_eq!(collision_from_link.name, "collision1");
}

#[test]
fn original_dom_link_add_visual() {
    let (mut link, _, visual) = link_and_elements();
    assert_eq!(link.visuals.len(), 0);
    assert!(link.add_visual_unique(visual));
    assert_eq!(link.visuals.len(), 1);
    let (_, _, duplicate) = link_and_elements();
    assert!(!link.add_visual_unique(duplicate));
    assert_eq!(link.visuals.len(), 1);

    link.clear_visuals();
    assert_eq!(link.visuals.len(), 0);
    let (_, _, visual) = link_and_elements();
    assert!(link.add_visual_unique(visual));
    assert_eq!(link.visuals.len(), 1);
    let visual_from_link = link.visuals.first().expect("VisualByIndex(0)");
    assert_eq!(visual_from_link.name, "visual1");
}
