// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
use sdformat::{Scene, Sdf};

pub fn assert_scene(sdf: &Sdf, xml: &str, uri: &str, configured: bool, light_count: usize) {
    assert_eq!(sdf.worlds.len(), 1);
    let w = &sdf.worlds[0];
    assert_eq!(w.name, "w");
    assert!(w.extensions.is_empty());
    assert!(w.models.is_empty() && w.frames.is_empty());
    assert_eq!(w.lights.len(), light_count);
    assert_eq!(w.wind_linear_velocity, [0.; 3]);
    assert_eq!(w.gravity, [0., 0., -9.8]);
    assert_eq!(
        w.scene.ambient,
        if configured {
            [0.1, 0.2, 0.3, 1.]
        } else {
            Scene::default().ambient
        }
    );
    assert_eq!(w.scene.background, Scene::default().background);
    assert!(w.scene.grid && w.scene.shadows && w.scene.origin_visual);
    let source = w.scene.source.as_ref().unwrap();
    assert_eq!(source.uri, uri);
    assert_eq!(source.line, 1);
    assert_eq!(source.column as usize, xml.find("<scene").unwrap() + 1);
    assert_eq!(w.source.uri, uri);
    assert_eq!(
        (w.source.line, w.source.column),
        (1, xml.find("<world").unwrap() as u32 + 1)
    );
}
