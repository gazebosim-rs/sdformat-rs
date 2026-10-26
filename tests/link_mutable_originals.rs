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
// Fixed observable values from DOMLink.MutableByIndex and MutableByName.
use sdformat::{Light, Link, NoResources, ParticleEmitter, Projector, load_str};

fn populated_link() -> Link {
    let mut link = load_str(
        "<sdf version='1.12'><model name='parent'><link name='my-name'><visual name='visual1'/><collision name='collision1'><geometry><sphere><radius>1</radius></sphere></geometry></collision><sensor name='sensor1' type='imu'><imu/></sensor></link></model></sdf>",
        "link-mutable.sdf",
        &NoResources,
    )
    .unwrap()
    .models
    .remove(0)
    .links
    .remove(0);
    let visual = link.visuals.remove(0);
    let collision = link.collisions.remove(0);
    let sensor = link.sensors.remove(0);

    assert!(link.add_visual_unique(visual));
    assert!(link.add_collision_unique(collision));
    assert!(link.add_light_unique(Light {
        name: "light1".into(),
    }));
    assert!(link.add_sensor_unique(sensor));
    assert!(link.add_particle_emitter_unique(ParticleEmitter { name: "pe1".into() }));
    assert!(link.add_projector_unique(Projector {
        name: "projector1".into(),
    }));
    link
}

#[test]
fn original_dom_link_mutable_by_index() {
    let mut link = populated_link();

    let visual = link.visuals.get_mut(0).expect("VisualByIndex(0)");
    assert_eq!(visual.name, "visual1");
    visual.name = "visual2".into();
    assert_eq!(link.visuals[0].name, "visual2");

    let collision = link.collisions.get_mut(0).expect("CollisionByIndex(0)");
    assert_eq!(collision.name, "collision1");
    collision.name = "collision2".into();
    assert_eq!(link.collisions[0].name, "collision2");

    let light = link.lights.get_mut(0).expect("LightByIndex(0)");
    assert_eq!(light.name, "light1");
    light.name = "light2".into();
    assert_eq!(link.lights[0].name, "light2");

    let sensor = link.sensors.get_mut(0).expect("SensorByIndex(0)");
    assert_eq!(sensor.name, "sensor1");
    sensor.name = "sensor2".into();
    assert_eq!(link.sensors[0].name, "sensor2");

    let emitter = link
        .particle_emitters
        .get_mut(0)
        .expect("ParticleEmitterByIndex(0)");
    assert_eq!(emitter.name, "pe1");
    emitter.name = "pe2".into();
    assert_eq!(link.particle_emitters[0].name, "pe2");

    let projector = link.projectors.get_mut(0).expect("ProjectorByIndex(0)");
    assert_eq!(projector.name, "projector1");
    projector.name = "projector2".into();
    assert_eq!(link.projectors[0].name, "projector2");
}

#[test]
fn original_dom_link_mutable_by_name() {
    let mut link = populated_link();

    let visual = link
        .visuals
        .iter_mut()
        .find(|member| member.name == "visual1")
        .expect("VisualByName(visual1)");
    assert_eq!(visual.name, "visual1");
    visual.name = "visual2".into();
    assert!(!link.visuals.iter().any(|member| member.name == "visual1"));
    assert!(link.visuals.iter().any(|member| member.name == "visual2"));

    let collision = link
        .collisions
        .iter_mut()
        .find(|member| member.name == "collision1")
        .expect("CollisionByName(collision1)");
    assert_eq!(collision.name, "collision1");
    collision.name = "collision2".into();
    assert!(
        !link
            .collisions
            .iter()
            .any(|member| member.name == "collision1")
    );
    assert!(
        link.collisions
            .iter()
            .any(|member| member.name == "collision2")
    );

    let light = link
        .lights
        .iter_mut()
        .find(|member| member.name == "light1")
        .expect("LightByName(light1)");
    assert_eq!(light.name, "light1");
    light.name = "light2".into();
    assert!(!link.lights.iter().any(|member| member.name == "light1"));
    assert!(link.lights.iter().any(|member| member.name == "light2"));

    let sensor = link
        .sensors
        .iter_mut()
        .find(|member| member.name == "sensor1")
        .expect("SensorByName(sensor1)");
    assert_eq!(sensor.name, "sensor1");
    sensor.name = "sensor2".into();
    assert!(!link.sensors.iter().any(|member| member.name == "sensor1"));
    assert!(link.sensors.iter().any(|member| member.name == "sensor2"));

    let emitter = link
        .particle_emitters
        .iter_mut()
        .find(|member| member.name == "pe1")
        .expect("ParticleEmitterByName(pe1)");
    assert_eq!(emitter.name, "pe1");
    emitter.name = "pe2".into();
    assert!(
        !link
            .particle_emitters
            .iter()
            .any(|member| member.name == "pe1")
    );
    assert!(
        link.particle_emitters
            .iter()
            .any(|member| member.name == "pe2")
    );

    let projector = link
        .projectors
        .iter_mut()
        .find(|member| member.name == "projector1")
        .expect("ProjectorByName(projector1)");
    assert_eq!(projector.name, "projector1");
    projector.name = "projector2".into();
    assert!(
        !link
            .projectors
            .iter()
            .any(|member| member.name == "projector1")
    );
    assert!(
        link.projectors
            .iter()
            .any(|member| member.name == "projector2")
    );
}

#[test]
fn supplemental_new_collections_reject_duplicate_direct_names() {
    let mut link = populated_link();
    assert!(!link.add_particle_emitter_unique(ParticleEmitter { name: "pe1".into() }));
    assert!(!link.add_projector_unique(Projector {
        name: "projector1".into(),
    }));
    assert_eq!(link.particle_emitters.len(), 1);
    assert_eq!(link.projectors.len(), 1);
}
