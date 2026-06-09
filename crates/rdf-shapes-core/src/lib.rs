//! Portable core logic for `rdf-shapes-wasm`.
//!
//! This crate holds all business logic and is compiled both for the
//! native host triple and for `wasm32-unknown-unknown`. It has no
//! dependency on `wasm-bindgen` and performs no I/O, threading,
//! filesystem, or socket access, so it stays portable across every
//! target.
//!
//! The heavy graph engines (Oxigraph for SPARQL, rudof for SHACL)
//! are introduced by the de-risking spikes, not by this scaffold.
//! For now the surface is intentionally trivial: a [`version`] and a
//! [`ping`] that prove the toolchain end to end.

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
