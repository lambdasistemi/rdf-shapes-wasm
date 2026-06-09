//! Portable core logic for `rdf-shapes-wasm`.
//!
//! This crate holds all business logic and is compiled both for the
//! native host triple and for `wasm32-unknown-unknown`. It has no
//! dependency on `wasm-bindgen` and performs no I/O, threading,
//! filesystem, or socket access, so it stays portable across every
//! target.
//!
//! The façade exposes two capabilities over in-memory Turtle:
//!
//! - [`query`] — full SPARQL 1.1 query via Oxigraph's in-memory store,
//!   returning typed [`QueryResults`].
//! - [`validate`] — SHACL Core validation via rudof, returning a
//!   structured [`ValidationReport`].
//!
//! Every fallible path returns a [`ShapesError`]; nothing panics.

mod error;
mod report;
mod results;

pub use error::ShapesError;
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

/// Echoes `input` back as a `pong:`-prefixed greeting.
///
/// This is the trivial round-trip used to prove that a value flows
/// from a caller, through the portable core, and back out — across
/// both the native CLI and the wasm shim.
#[must_use]
pub fn ping(input: &str) -> String {
    format!("pong: {input}")
}

#[cfg(test)]
mod tests {
    use super::{ping, version};

    #[test]
    fn version_matches_cargo_metadata() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn ping_prefixes_with_pong() {
        assert_eq!(ping("hello"), "pong: hello");
    }
}
