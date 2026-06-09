//! Thin `wasm-bindgen` shell over [`rdf_shapes_core`].
//!
//! This crate is a `cdylib` carrying only the `#[wasm_bindgen]`
//! shims that marshal values to and from the portable core. It adds
//! no business logic of its own — every exported function is a direct
//! delegation to `rdf-shapes-core`.

use wasm_bindgen::prelude::wasm_bindgen;

/// Installs a panic hook that forwards Rust panics to `console.error`.
///
/// Call this once from JavaScript after instantiating the module to
/// get readable stack traces instead of an opaque `unreachable` trap.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Returns the crate version, delegating to [`rdf_shapes_core::version`].
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    rdf_shapes_core::version().to_owned()
}

/// Echoes `input` back, delegating to [`rdf_shapes_core::ping`].
#[wasm_bindgen]
#[must_use]
pub fn ping(input: &str) -> String {
    rdf_shapes_core::ping(input)
}
