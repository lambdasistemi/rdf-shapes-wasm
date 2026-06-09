//! Thin `wasm-bindgen` shell over [`rdf_shapes_core`].
//!
//! This crate is a `cdylib` carrying only the `#[wasm_bindgen]`
//! shims that marshal values to and from the portable core. It adds
//! no business logic of its own — every exported function is a direct
//! delegation to `rdf-shapes-core`.

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsError;

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

/// SPIKE (#3): runs `sparql` over the in-memory graph parsed from
/// `turtle_data`, returning the JSON result string.
///
/// Delegates to [`rdf_shapes_core::query`]; on error the message is
/// surfaced to JavaScript as a thrown `Error`.
///
/// # Errors
///
/// Returns the error from [`rdf_shapes_core::query`] (Turtle load,
/// SPARQL parse, or evaluation failure) as a JS `Error`.
#[wasm_bindgen]
pub fn query(turtle_data: &str, sparql: &str) -> Result<String, JsError> {
    rdf_shapes_core::query(turtle_data, sparql)
        .map_err(|e| JsError::new(&e.to_string()))
}
