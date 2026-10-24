//! Validated SDFormat model and frame semantics, independent of a simulator.
//!
//! Loading is atomic: syntax, validation, include expansion and graph errors never
//! expose a partially validated model. Unsupported schema features are errors.
mod graph;
mod parse;
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

/// Parse the supported SDF 1.7–1.12 subset and validate its semantic graphs.
pub fn load_str(
    xml: &str,
    source: &str,
    resolver: &dyn ResourceResolver,
) -> Result<Sdf, Diagnostic> {
    parse::load(xml, source, resolver)
}
