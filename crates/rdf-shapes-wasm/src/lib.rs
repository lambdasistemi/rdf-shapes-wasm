//! Thin `wasm-bindgen` shell over [`rdf_shapes_core`].
//!
//! This crate is a `cdylib` carrying only the `#[wasm_bindgen]` shims
//! that marshal values to and from the portable core. It adds no
//! business logic of its own: each exported function calls into
//! `rdf-shapes-core`, serializes the typed result to a `JsValue` via
//! `serde-wasm-bindgen`, and maps a [`rdf_shapes_core::ShapesError`] to
//! a thrown JS `Error`.

use serde::Serialize;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/// Serializes a core result to a plain JS object.
///
/// Uses `serde-wasm-bindgen`'s json-compatible serializer so serde
/// maps (e.g. the `serde_json::Value` inside a SELECT result) become
/// plain JS objects rather than `Map`s — otherwise nested JSON would
/// read back as an empty object on the JS side.
fn to_js(value: &impl Serialize) -> Result<JsValue, JsValue> {
    value.serialize(&Serializer::json_compatible()).map_err(|e| JsValue::from_str(&e.to_string()))
}

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

/// Runs `sparql` over the graph parsed from `graph_ttl`, returning the
/// typed query results as a JS object.
///
/// The result mirrors [`rdf_shapes_core::QueryResults`]: a `kind` tag
/// plus the SELECT Query Results JSON, the ASK boolean, or the
/// CONSTRUCT/DESCRIBE N-Triples.
///
/// # Errors
///
/// Returns the [`rdf_shapes_core::ShapesError`] (Turtle load, SPARQL
/// parse, or evaluation failure) as a thrown JS `Error`.
#[wasm_bindgen]
pub fn query(graph_ttl: &str, sparql: &str) -> Result<JsValue, JsValue> {
    let results =
        rdf_shapes_core::query(graph_ttl, sparql).map_err(|e| JsValue::from_str(&e.to_string()))?;
    to_js(&results)
}

/// Validates `data_ttl` against `shapes_ttl` (SHACL Core), returning
/// the conformance report as a JS object.
///
/// The result mirrors [`rdf_shapes_core::ValidationReport`]: a
/// `conforms` flag plus a `violations` array.
///
/// # Errors
///
/// Returns the [`rdf_shapes_core::ShapesError`] (parse, validation, or
/// an unsupported SHACL-SPARQL / remote-graph construct) as a thrown
/// JS `Error`.
#[wasm_bindgen]
pub fn validate(data_ttl: &str, shapes_ttl: &str) -> Result<JsValue, JsValue> {
    let report = rdf_shapes_core::validate(data_ttl, shapes_ttl)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    to_js(&report)
}
