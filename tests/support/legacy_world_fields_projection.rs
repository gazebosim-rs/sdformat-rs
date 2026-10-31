// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0
pub fn assert_gravity_conversion(sdf: &sdformat::Sdf) {
    use sdformat::{AtmosphereType, Physics, Scene, Source, World};
    assert_eq!(sdf.version, "1.5");
    assert!(sdf.models.is_empty());
    assert_eq!(sdf.worlds.len(), 1);
    let World {
        name,
        gravity,
        wind_linear_velocity,
        magnetic_field,
        atmosphere,
        scene,
        gui,
        audio_device,
        spherical_coordinates,
        physics,
        models,
        lights,
        joints,
        frames,
        merged_includes,
        plugins,
        extensions,
        source,
    } = &sdf.worlds[0];
    assert_eq!(name, "w");
    assert_eq!(*gravity, [1., 2., 3.]);
    assert_eq!(*wind_linear_velocity, [0.; 3]);
    assert_eq!(*magnetic_field, [5.5645e-6, 22.8758e-6, -42.3884e-6]);
    assert_eq!(atmosphere.atmosphere_type, AtmosphereType::Adiabatic);
    assert_eq!(atmosphere.temperature_kelvin, 288.15);
    assert_eq!(atmosphere.temperature_gradient, -0.0065);
    assert_eq!(atmosphere.pressure, 101325.);
    assert_eq!(atmosphere.source, None);
    assert_eq!(atmosphere.xml, None);
    assert_eq!(*scene, Scene::default());
    assert!(gui.is_none());
    assert_eq!(audio_device, "default");
    assert_eq!(*spherical_coordinates, None);
    assert_eq!(
        *physics,
        [Physics {
            name: "default_physics".into(),
            source: Some(Source {
                uri: "unsupported-conversion".into(),
                line: 1,
                column: 36,
            }),
            xml: Some("<physics type='ode'><gravity>1 2 3</gravity></physics>".into()),
            ..Physics::default()
        }]
    );
    assert!(std::ptr::eq(
        sdf.worlds[0].physics_default().unwrap(),
        &physics[0]
    ));
    assert!(models.is_empty());
    assert!(lights.is_empty());
    assert!(joints.is_empty());
    assert!(frames.is_empty());
    assert!(merged_includes.is_empty());
    assert!(plugins.is_empty());
    assert!(extensions.is_empty());
    assert_eq!(
        *source,
        Source {
            uri: "unsupported-conversion".into(),
            line: 1,
            column: 20,
        }
    );
    sdf.validate_non_frame_poses().unwrap();
}
