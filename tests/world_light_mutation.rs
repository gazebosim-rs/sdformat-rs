/*
 * Copyright (C) 2017 Open Source Robotics Foundation
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
// The first test preserves every observable assertion from DOMWorld.AddLight.
use sdformat::{Light, NoResources, World, load_str};

fn empty_world() -> World {
    load_str(
        "<sdf version='1.12'><world name='w'/></sdf>",
        "world-add-light.sdf",
        &NoResources,
    )
    .unwrap()
    .worlds
    .remove(0)
}

#[test]
fn original_dom_world_add_light() {
    let mut world = empty_world();
    assert_eq!(world.lights.len(), 0);
    let light = Light {
        name: "light1".into(),
        ..Light::default()
    };
    assert!(world.add_light_unique(light.clone()));
    assert_eq!(world.lights.len(), 1);
    assert!(!world.add_light_unique(light.clone()));
    assert_eq!(world.lights.len(), 1);
    world.clear_lights();
    assert_eq!(world.lights.len(), 0);
    assert!(world.add_light_unique(light.clone()));
    assert_eq!(world.lights.len(), 1);
    let light_from_world = world.lights.first().expect("LightByIndex(0)");
    assert_eq!(light_from_world.name, light.name);
}

#[test]
fn direct_names_only_preserve_first_value_and_reinsertion() {
    for name in ["", "light", "Light", "__world__", "a::b", "a b"] {
        let mut world = empty_world();
        let mut light = Light {
            name: name.into(),
            intensity: 3.,
            ..Light::default()
        };
        light.pose.transform.translation = [1., 2., 3.];
        let expected = light.clone();
        assert!(world.add_light_unique(light.clone()));
        assert_eq!(world.lights.len(), 1);
        light.intensity = 9.;
        assert!(!world.add_light_unique(light.clone()));
        assert_eq!(world.lights, vec![expected]);
        world.clear_lights();
        assert!(world.lights.is_empty());
        assert_eq!(world.lights.first(), None);
        assert!(world.add_light_unique(light.clone()));
        assert_eq!(world.lights, vec![light]);
    }
}

#[test]
fn rename_order_clone_and_loaded_source_are_owned() {
    let mut sdf = load_str(
        "<sdf version='1.12'><world name='w'><frame name='reference'><pose>1 2 3 0 0 0</pose></frame><light name='light' type='spot'><pose relative_to='reference'>4 5 6 0 0 0</pose><intensity>3</intensity><diffuse>0.1 0.2 0.3 0.4</diffuse><spot><inner_angle>0.2</inner_angle><outer_angle>0.4</outer_angle><falloff>2</falloff></spot></light></world></sdf>",
        "owned-world-light.sdf",
        &NoResources,
    )
    .unwrap();
    let graph_before = sdf.pose_of("w::reference", "::").unwrap();
    let world_source = sdf.worlds[0].source.clone();
    let world = &mut sdf.worlds[0];
    let first = world.lights[0].clone();
    assert_eq!(first.source.as_ref().unwrap().uri, "owned-world-light.sdf");
    let mut second = first.clone();
    second.name = "Light".into();
    assert!(world.add_light_unique(second.clone()));
    assert_eq!(world.lights, vec![first.clone(), second]);
    world.lights[0].name = "renamed".into();
    assert!(!world.lights.iter().any(|light| light.name == "light"));
    assert!(world.lights.iter().any(|light| light.name == "renamed"));
    assert!(world.add_light_unique(first.clone()));
    let mut duplicate = first.clone();
    duplicate.name = "renamed".into();
    duplicate.intensity = 12.;
    assert!(!world.add_light_unique(duplicate));
    assert_eq!(
        world
            .lights
            .iter()
            .map(|light| light.name.as_str())
            .collect::<Vec<_>>(),
        ["renamed", "Light", "light"]
    );
    let retained = world.lights.clone();
    let mut copy = empty_world();
    copy.lights = world.lights.clone();
    copy.lights[0].source.as_mut().unwrap().uri = "copy-only.sdf".into();
    copy.lights[0].intensity = 20.;
    copy.clear_lights();
    assert_eq!(world.lights, retained);
    assert_eq!(world.source, world_source);
    world.clear_lights();
    world.clear_lights();
    assert!(world.lights.is_empty());
    assert!(world.add_light_unique(first.clone()));
    assert_eq!(world.lights, vec![first]);
    assert_eq!(sdf.pose_of("w::reference", "::").unwrap(), graph_before);
}
