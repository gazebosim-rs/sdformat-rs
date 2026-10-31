// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
pub fn assert_spherical(sdf: &sdformat::Sdf, uri: &str) {
    use sdformat::{AtmosphereType, Physics, Scene, SphericalSurface, WorldFrameOrientation};
    assert_eq!(sdf.version, "1.12");
    assert_eq!(sdf.worlds.len(), 1);
    assert!(sdf.models.is_empty());
    let world = &sdf.worlds[0];
    assert_eq!(world.name, "w");
    assert_eq!(world.source.uri, uri);
    assert_eq!((world.source.line, world.source.column), (1, 21));
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
    assert_eq!(
        world.physics,
        [Physics {
            name: "default_physics".into(),
            ..Physics::default()
        }]
    );
    assert!(world.models.is_empty());
    assert!(world.lights.is_empty());
    assert!(world.joints.is_empty());
    assert!(world.frames.is_empty());
    assert!(world.merged_includes.is_empty());
    assert!(world.plugins.is_empty());
    assert!(world.extensions.is_empty());
    let value = world.spherical_coordinates.as_ref().unwrap();
    assert_eq!(value.surface, SphericalSurface::EarthWgs84);
    assert_eq!(value.orientation, WorldFrameOrientation::EastNorthUp);
    assert_eq!(
        [
            value.latitude_radians,
            value.longitude_radians,
            value.heading_radians,
            value.elevation
        ]
        .map(f64::to_bits),
        [0; 4]
    );
    assert_eq!(value.source.uri, uri);
    assert_eq!((value.source.line, value.source.column), (1, 37));
}
