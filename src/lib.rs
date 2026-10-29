// Copyright (C) 2026 Open Source Robotics Foundation
// SPDX-License-Identifier: Apache-2.0

//! Validated SDFormat model and frame semantics, independent of a simulator.
//!
//! Loading returns an owned document or a source-located error. Frame graphs are
//! validated during loading; native non-frame pose errors may remain deferred.
//! Before instantiation, call [`Sdf::validate_non_frame_poses`] to validate those
//! references, including the documented stronger inertial-frame policy.
//! Unsupported schema features are errors.
mod axis_xml;
mod camera;
mod frame_xml;
mod graph;
mod light_xml;
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
