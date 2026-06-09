//! Portable core logic for `rdf-shapes-wasm`.
//!
//! This crate holds all business logic and is compiled both for the
//! native host triple and for `wasm32-unknown-unknown`. It has no
//! dependency on `wasm-bindgen` and performs no I/O, threading,
//! filesystem, or socket access, so it stays portable across every
//! target.
//!
//! The façade exposes full SPARQL 1.1 query via Oxigraph's in-memory
//! store ([`query`], returning typed [`QueryResults`]); SHACL Core
//! validation via rudof lands alongside it. Every fallible path
//! returns a [`ShapesError`]; nothing panics.

mod error;
mod query;
mod report;
mod results;

pub use error::ShapesError;
pub use query::query;
pub use report::{ValidationReport, Violation};
pub use results::QueryResults;

/// Returns the crate version, taken from the Cargo package metadata.
///
/// This is the single source of truth threaded through the `wasm`
/// and `cli` shells so every artifact reports the same version.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version;

    #[test]
    fn version_matches_cargo_metadata() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
