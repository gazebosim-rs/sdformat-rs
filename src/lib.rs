// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

//! Validated SDFormat model and frame semantics, independent of a simulator.
//!
//! Loading returns an owned document or a source-located error. Frame graphs are
//! validated during loading; native non-frame pose errors may remain deferred.
//! Before instantiation, call [`Sdf::validate_non_frame_poses`] to validate those
//! references, including the documented stronger inertial-frame policy.
//! Unsupported schema features are errors.
mod air_pressure_xml;
mod air_speed_xml;
mod altimeter_xml;
mod axis_xml;
mod camera;
mod camera_xml;
mod force_torque_xml;
mod frame_xml;
mod graph;
mod imu_xml;
mod joint_xml;
mod light_xml;
mod magnetometer_xml;
mod navsat_xml;
mod noise_xml;
pub use camera::*;
mod model_config;
pub use model_config::{
    ModelConfigSelection, ModelConfigWarning, ModelConfigWarningKind, select_model_config,
};
// The only owned production unsafe boundary; see docs/SCALAR_CONVERSION.md.
#[allow(unsafe_code)]
mod number;
mod parse;
mod physics_xml;
mod sensor_xml;
mod types;
mod xml;
pub use types::*;

/// Resolve include URIs without introducing a transport or cache dependency.
/// Implementations control URI policy and retain their canonical resource identity.
pub trait ResourceResolver {
    fn resolve(&self, uri: &str, parent: &str) -> Result<Resource, String>;
}

/// A resolver suitable for self-contained documents; includes fail explicitly.
pub struct NoResources;
impl ResourceResolver for NoResources {
    fn resolve(&self, uri: &str, _: &str) -> Result<Resource, String> {
        Err(format!("no resource resolver for {uri}"))
    }
}

/// Parse the supported SDF 1.4–1.12 subset and validate its frame graphs.
/// Call [`Sdf::validate_non_frame_poses`] before constructing runtime objects.
pub fn load_str(
    xml: &str,
    source: &str,
    resolver: &dyn ResourceResolver,
) -> Result<Sdf, Diagnostic> {
    parse::load(xml, source, resolver)
}

/// Load one bounded standalone `<light>` element into owned DOM values.
/// The full SDF document loader does not yet admit Light XML.
pub fn load_light_str(xml: &str, source: &str) -> Result<Light, Diagnostic> {
    parse::load_light_element(xml, source)
}

/// Load one bounded standalone `<frame>` element into owned DOM values.
/// The full SDF loader applies frame graph rules separately.
pub fn load_frame_str(xml: &str, source: &str) -> Result<Frame, Diagnostic> {
    parse::load_frame_element(xml, source)
}

/// Load current-schema standalone axis metadata without resolving graph references.
/// Model loading validates frame and mimic references separately. XML axis2 roots
/// and legacy conversion directives are outside this standalone interface.
pub fn load_axis_str(xml: &str, source: &str) -> Result<Axis, Diagnostic> {
    parse::load_axis_element(xml, source)
}

/// Load bounded current-schema joint metadata without attaching a frame graph.
/// Sensors/physics payloads and unrepresentable axis/pitch configurations return
/// explicit errors. Containing model loading performs graph validation.
pub fn load_joint_str(xml: &str, source: &str) -> Result<Joint, Diagnostic> {
    parse::load_joint_element(xml, source)
}

/// Load unattached current-schema force/torque configuration. Existing finite
/// noise admission applies; no sensor envelope or frame graph is created.
pub fn load_force_torque_str(xml: &str, source: &str) -> Result<ForceTorque, Diagnostic> {
    parse::load_force_torque_element(xml, source)
}

/// Load unattached current-schema IMU configuration without resolving its frames.
/// Existing finite-noise admission remains in force.
pub fn load_imu_str(xml: &str, source: &str) -> Result<Imu, Diagnostic> {
    parse::load_imu_element(xml, source)
}

/// Load unattached altimeter noise configuration using existing finite-noise rules.
/// No height, velocity or sensor graph is computed.
pub fn load_altimeter_str(xml: &str, source: &str) -> Result<Altimeter, Diagnostic> {
    parse::load_altimeter_element(xml, source)
}

/// Load unattached pressure-noise and reference-altitude metadata.
/// Existing finite-noise rules apply; no atmosphere or sensor graph is evaluated.
pub fn load_air_pressure_str(xml: &str, source: &str) -> Result<AirPressure, Diagnostic> {
    parse::load_air_pressure_element(xml, source)
}

/// Load unattached magnetometer Noise metadata with existing finite-noise rules.
/// No magnetic field or sensor graph is computed.
pub fn load_magnetometer_str(xml: &str, source: &str) -> Result<Magnetometer, Diagnostic> {
    parse::load_magnetometer_element(xml, source)
}

/// Load canonical unattached `<navsat>` Noise configuration using existing rules.
/// Legacy `<gps>` remains supported by document parsing, outside this root API.
pub fn load_navsat_str(xml: &str, source: &str) -> Result<NavSat, Diagnostic> {
    parse::load_navsat_element(xml, source)
}

/// Load unattached AirSpeed pressure Noise with existing finite-noise rules.
/// No pressure-to-speed conversion or atmosphere is evaluated.
pub fn load_air_speed_str(xml: &str, source: &str) -> Result<AirSpeed, Diagnostic> {
    parse::load_air_speed_element(xml, source)
}

/// Load bounded current-schema base-camera XML. Unsupported advanced payloads
/// return explicit errors; no sensor, graph or renderer is created.
pub fn load_camera_str(xml: &str, source: &str) -> Result<Camera, Diagnostic> {
    parse::load_camera_element(xml, source)
}
