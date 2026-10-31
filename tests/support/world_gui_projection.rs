// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
#[path = "document_light_projection.rs"]
pub(crate) mod light;

pub fn assert_gui(
    sdf: &sdformat::Sdf,
    xml: &str,
    uri: &str,
    version: &str,
    fullscreen: bool,
    native_light: Option<&str>,
) {
    use sdformat::{AtmosphereType, Physics, Scene};
    assert_eq!(sdf.version, version);
    assert_eq!(sdf.worlds.len(), 1);
    assert!(sdf.models.is_empty());
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "w");
    assert_eq!(world.source.uri, uri);
    assert_eq!(
        (world.source.line, world.source.column),
        (1, xml.find("<world").unwrap() as u32 + 1)
    );
    assert_eq!(world.audio_device, "default");
    assert_eq!(world.gravity, [0., 0., -9.8]);
    assert_eq!(world.wind_linear_velocity, [0.; 3]);
    assert_eq!(world.magnetic_field, [5.5645e-6, 22.8758e-6, -42.3884e-6]);
    assert_eq!(world.atmosphere.atmosphere_type, AtmosphereType::Adiabatic);
    assert_eq!(world.atmosphere.temperature_kelvin, 288.15);
    assert_eq!(world.atmosphere.temperature_gradient, -0.0065);
    assert_eq!(world.atmosphere.pressure, 101325.);
    assert_eq!(world.atmosphere.source, None);
    assert_eq!(world.atmosphere.xml, None);
    assert_eq!(world.scene, Scene::default());
    assert_eq!(world.spherical_coordinates, None);
    assert_eq!(
        world.physics,
        [Physics {
            name: "default_physics".into(),
            ..Physics::default()
        }]
    );
    assert!(world.models.is_empty());
    assert!(world.joints.is_empty());
    assert!(world.frames.is_empty());
    assert!(world.merged_includes.is_empty());
    assert!(world.plugins.is_empty());
    assert!(world.extensions.is_empty());
    let gui = world.gui.as_ref().unwrap();
    assert_eq!(gui.fullscreen, fullscreen);
    assert!(gui.plugins.is_empty());
    assert_eq!(gui.source.uri, uri);
    assert_eq!(
        (gui.source.line, gui.source.column),
        (1, xml.find("<gui").unwrap() as u32 + 1)
    );
    assert_eq!(
        gui.xml,
        if fullscreen {
            "<gui fullscreen='true'/>"
        } else {
            "<gui/>"
        }
    );
    assert_eq!(world.lights.len(), usize::from(native_light.is_some()));
    if let Some(native) = native_light {
        let rows: Vec<Vec<String>> = native
            .lines()
            .map(|line| line.split_whitespace().map(str::to_owned).collect())
            .collect();
        assert_eq!(rows.len(), 2);
        light::compare(&light::projection(&world.lights[0]), &rows[0]);
        assert_eq!(&rows[1][..3], ["POSE", "default", "OK"]);
        light::pose(
            sdf.resolve_pose(&world.lights[0].pose, "w", "w", "w")
                .unwrap(),
            &rows[1][3..],
        );
        let source = world.lights[0].source.as_ref().unwrap();
        assert_eq!(source.uri, uri);
        assert_eq!(
            (source.line, source.column),
            (1, xml.find("<light").unwrap() as u32 + 1)
        );
        sdf.validate_non_frame_poses().unwrap();
    }
}
