/*
 * Copyright 2019 Open Source Robotics Foundation
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
 */
// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::*;

fn common(flow: &PbrWorkflow, kind: PbrWorkflowKind, normal: &str, uv_set: u32) {
    assert_eq!(flow.kind, kind);
    assert_eq!(flow.albedo_map, "albedo_map.png");
    assert_eq!(flow.normal_map, normal);
    assert_eq!(flow.environment_map, "environment_map.png");
    assert_eq!(flow.ambient_occlusion_map, "ambient_occlusion_map.png");
    assert_eq!(flow.emissive_map, "emissive_map.png");
    assert_eq!(flow.light_map, "light_map.png");
    assert_eq!(flow.light_map_uv_set, uv_set);
}
fn metal(flow: &PbrWorkflow) {
    assert_eq!(flow.metalness_map, "metalness_map.png");
    assert_eq!(flow.metalness.to_bits(), 0.3_f64.to_bits());
    assert_eq!(flow.roughness_map, "roughness_map.png");
    assert_eq!(flow.roughness.to_bits(), 0.4_f64.to_bits());
}
fn specular(flow: &PbrWorkflow) {
    assert_eq!(flow.glossiness_map, "glossiness_map.png");
    assert_eq!(flow.glossiness.to_bits(), 0.2_f64.to_bits());
    assert_eq!(flow.specular_map, "specular_map.png");
}
#[test]
fn original_material_pbr_dom() {
    let sdf = load_str(
        include_str!("fixtures/material_pbr.sdf"),
        "material_pbr.sdf",
        &NoResources,
    )
    .unwrap();
    let model = sdf.models.first().expect("Root.Model() is present");
    let link = model
        .links
        .first()
        .expect("Model.LinkByIndex(0) is present");
    assert_eq!(link.name, "link");
    assert_eq!(link.visuals.len(), 3);
    let names = [
        "visual_metal_workflow",
        "visual_specular_workflow",
        "visual_all",
    ];
    for (index, name) in names.into_iter().enumerate() {
        assert!(link.visuals.iter().any(|visual| visual.name == name));
        let visual = link.visuals.get(index).unwrap();
        assert_eq!(visual.name, name);
        let material = visual.material.as_ref().unwrap();
        for (actual, expected) in material.diffuse.into_iter().zip([0.2, 0.5, 0.1, 1.]) {
            assert!((actual - expected).abs() <= 1e-6_f32);
        }
        let pbr = material.pbr.as_ref().unwrap();
        match index {
            0 => {
                let flow = pbr.metal.as_ref().unwrap();
                assert!(pbr.specular.is_none());
                common(flow, PbrWorkflowKind::Metal, "normal_map.png", 1);
                metal(flow);
            }
            1 => {
                let flow = pbr.specular.as_ref().unwrap();
                assert!(pbr.metal.is_none());
                common(flow, PbrWorkflowKind::Specular, "normal_map.png", 2);
                specular(flow);
            }
            2 => {
                let flow = pbr.specular.as_ref().unwrap();
                common(flow, PbrWorkflowKind::Specular, "normal_map_tangent.png", 3);
                // The original repeats the specular Type assertion here.
                assert_eq!(flow.kind, PbrWorkflowKind::Specular);
                specular(flow);
                let flow = pbr.metal.as_ref().unwrap();
                common(flow, PbrWorkflowKind::Metal, "normal_map_object.png", 0);
                metal(flow);
            }
            _ => unreachable!(),
        }
    }
}
