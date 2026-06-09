//! Native C-ABI shell over [`rdf_shapes_core`].
//!
//! This crate is a `cdylib` carrying only the `extern "C"` shims that
//! marshal C strings to and from the portable core. It is the native
//! sibling of `rdf-shapes-wasm`: where the wasm crate hands the engine
//! to the browser through `wasm-bindgen`, this crate hands the same
//! engine to a native host — the Haskell backend (`amaru-treasury-tx`,
//! replacing Jena `arq`/`shacl`) via `foreign import ccall`, or any C
//! ABI consumer.
//!
//! # The C ABI contract
//!
//! Three computing functions and one deallocator are exported:
//!
//! - [`rdf_shapes_query`] — run a SPARQL 1.1 query over a Turtle graph.
//! - [`rdf_shapes_validate`] — SHACL Core validation of a data graph
//!   against a shapes graph.
//! - [`rdf_shapes_version`] — the engine version string.
//! - [`rdf_shapes_string_free`] — free a string returned by any of the
//!   above.
//!
//! Every computing function takes borrowed, NUL-terminated UTF-8 C
//! strings as input (the caller keeps ownership of those) and returns a
//! freshly allocated, NUL-terminated UTF-8 **JSON envelope** that the
//! caller owns. The envelope is one of:
//!
//! - `{"ok": <result>}` on success — `<result>` is the serialized
//!   [`rdf_shapes_core::QueryResults`] /
//!   [`rdf_shapes_core::ValidationReport`], or the bare version string;
//! - `{"error": "<message>"}` on failure — `<message>` is the
//!   `Display` of the [`rdf_shapes_core::ShapesError`] (or a marshalling
//!   error, e.g. a NULL or non-UTF-8 input).
//!
//! A NULL input pointer or invalid UTF-8 yields an `{"error": ...}`
//! envelope, never undefined behaviour. The functions never panic
//! across the boundary; the workspace `release` profile builds with
//! `panic = "abort"`, so even an unexpected internal panic aborts the
//! process rather than unwinding into C.
//!
//! # Ownership
//!
//! Each returned `*mut c_char` is heap-allocated by Rust
//! ([`CString::into_raw`]) and **the caller MUST return it to**
//! [`rdf_shapes_string_free`] exactly once to release it. Freeing with
//! `libc::free`, freeing twice, using the pointer after freeing, or
//! leaking it are all contract violations. Passing a pointer not
//! obtained from one of the computing functions to
//! [`rdf_shapes_string_free`] is undefined behaviour.

use std::ffi::{c_char, CStr, CString};

use serde_json::json;

/// Builds a heap-allocated, NUL-terminated JSON envelope string and
/// transfers ownership to the caller.
///
/// `serde_json::to_string` over a JSON value cannot fail, and the
/// rendered JSON never contains an interior NUL byte, so the
/// `CString::new` is infallible in practice; the fallback keeps the
/// function total without an `unwrap`.
fn into_envelope(value: &serde_json::Value) -> *mut c_char {
    let text = serde_json::to_string(value)
        .unwrap_or_else(|_| r#"{"error":"failed to serialize envelope"}"#.to_owned());
    match CString::new(text) {
        Ok(c) => c.into_raw(),
        Err(_) => CString::new(r#"{"error":"envelope contained an interior NUL byte"}"#)
            .expect("static error string has no NUL")
            .into_raw(),
    }
}

/// An `{"ok": <result>}` success envelope over a serializable value.
fn ok_envelope(result: &impl serde::Serialize) -> *mut c_char {
    match serde_json::to_value(result) {
        Ok(value) => into_envelope(&json!({ "ok": value })),
        Err(e) => error_envelope(&format!("serialize result: {e}")),
    }
}

/// An `{"error": <message>}` failure envelope over a message.
fn error_envelope(message: &str) -> *mut c_char {
    into_envelope(&json!({ "error": message }))
}

/// Borrows a C string argument as `&str`, mapping NULL and non-UTF-8 to
/// a descriptive error so the caller never triggers UB.
///
/// # Safety
///
/// `ptr` must be either NULL or a valid pointer to a NUL-terminated C
/// string that stays alive for the duration of the call.
unsafe fn arg<'a>(ptr: *const c_char, name: &str) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err(format!("{name} argument was NULL"));
    }
    // SAFETY: `ptr` is non-NULL and the caller guarantees it points at a
    // NUL-terminated string valid for the call (see the fn-level
    // contract). The returned `&str` borrows that memory for `'a`.
    let bytes = unsafe { CStr::from_ptr(ptr) };
    bytes.to_str().map_err(|e| format!("{name} argument was not valid UTF-8: {e}"))
}

/// Runs `sparql` over the graph parsed from `graph`, returning a JSON
/// envelope.
///
/// On success the envelope is `{"ok": <QueryResults>}`, where
/// `<QueryResults>` is the serialized [`rdf_shapes_core::QueryResults`]
/// (a `kind`-tagged object: the SELECT Query Results JSON, the ASK
/// boolean, or the CONSTRUCT/DESCRIBE N-Triples). On any failure —
/// including a NULL or non-UTF-8 argument — the envelope is
/// `{"error": <message>}`.
///
/// # Safety
///
/// `graph` and `sparql` must each be NULL or a valid pointer to a
/// NUL-terminated UTF-8 C string. The returned pointer is owned by the
/// caller and MUST be released with [`rdf_shapes_string_free`].
#[no_mangle]
pub unsafe extern "C" fn rdf_shapes_query(
    graph: *const c_char,
    sparql: *const c_char,
) -> *mut c_char {
    // SAFETY: forwarding the caller's contract on `graph`/`sparql`.
    let parsed = unsafe { (arg(graph, "graph"), arg(sparql, "sparql")) };
    let (graph, sparql) = match parsed {
        (Ok(g), Ok(s)) => (g, s),
        (Err(e), _) | (_, Err(e)) => return error_envelope(&e),
    };
    match rdf_shapes_core::query(graph, sparql) {
        Ok(results) => ok_envelope(&results),
        Err(e) => error_envelope(&e.to_string()),
    }
}

/// Validates `data` against `shapes` (SHACL Core), returning a JSON
/// envelope.
///
/// On success the envelope is `{"ok": <ValidationReport>}`, where
/// `<ValidationReport>` is the serialized
/// [`rdf_shapes_core::ValidationReport`] (a `conforms` flag plus a
/// `violations` array). On any failure — including a NULL or non-UTF-8
/// argument, or an unsupported SHACL-SPARQL / remote-graph construct —
/// the envelope is `{"error": <message>}`.
///
/// # Safety
///
/// `data` and `shapes` must each be NULL or a valid pointer to a
/// NUL-terminated UTF-8 C string. The returned pointer is owned by the
/// caller and MUST be released with [`rdf_shapes_string_free`].
#[no_mangle]
pub unsafe extern "C" fn rdf_shapes_validate(
    data: *const c_char,
    shapes: *const c_char,
) -> *mut c_char {
    // SAFETY: forwarding the caller's contract on `data`/`shapes`.
    let parsed = unsafe { (arg(data, "data"), arg(shapes, "shapes")) };
    let (data, shapes) = match parsed {
        (Ok(d), Ok(s)) => (d, s),
        (Err(e), _) | (_, Err(e)) => return error_envelope(&e),
    };
    match rdf_shapes_core::validate(data, shapes) {
        Ok(report) => ok_envelope(&report),
        Err(e) => error_envelope(&e.to_string()),
    }
}

/// Returns the engine version as an `{"ok": "<version>"}` JSON
/// envelope.
///
/// # Safety
///
/// Takes no arguments. The returned pointer is owned by the caller and
/// MUST be released with [`rdf_shapes_string_free`].
#[no_mangle]
pub extern "C" fn rdf_shapes_version() -> *mut c_char {
    ok_envelope(&rdf_shapes_core::version())
}

/// Frees a string returned by [`rdf_shapes_query`],
/// [`rdf_shapes_validate`], or [`rdf_shapes_version`].
///
/// Passing NULL is a no-op. Each non-NULL pointer obtained from a
/// computing function MUST be passed here exactly once.
///
/// # Safety
///
/// `s` must be either NULL or a pointer previously returned by one of
/// this crate's computing functions and not yet freed. Passing any
/// other pointer, or the same pointer twice, is undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn rdf_shapes_string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    // SAFETY: by the contract `s` was produced by `CString::into_raw`
    // in one of the computing functions and has not been freed; retake
    // ownership so the `CString` drop releases it.
    let _ = unsafe { CString::from_raw(s) };
}

#[cfg(test)]
mod tests {
    use super::{
        rdf_shapes_query, rdf_shapes_string_free, rdf_shapes_validate, rdf_shapes_version,
    };
    use std::ffi::{c_char, CStr, CString};

    const GRAPH: &str = r#"
@prefix cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#> .
<urn:tx:aaaa> a cardano:Transaction .
<urn:tx:bbbb> a cardano:Transaction .
<urn:tx:cccc> a cardano:Transaction .
<urn:out:1>   a cardano:Output .
"#;

    const TX_COUNT: &str = r#"
PREFIX cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#>
SELECT (COUNT(DISTINCT ?tx) AS ?transactions)
WHERE { ?tx a cardano:Transaction . }
"#;

    const SHAPES: &str = r#"
@prefix atx: <https://lambdasistemi.github.io/amaru-treasury-tx/vocab/history#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
atx:HistoryEntryShape a sh:NodeShape ;
  sh:targetClass atx:HistoryEntry ;
  sh:property [ sh:path atx:slot ; sh:minCount 1 ; sh:datatype xsd:integer ] .
"#;

    const CONFORMING: &str = r#"
@prefix atx: <https://lambdasistemi.github.io/amaru-treasury-tx/vocab/history#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
atx:entry a atx:HistoryEntry ; atx:slot "42"^^xsd:integer .
"#;

    const VIOLATING: &str = r#"
@prefix atx: <https://lambdasistemi.github.io/amaru-treasury-tx/vocab/history#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
atx:entry a atx:HistoryEntry ; atx:slot "nope"^^xsd:string .
"#;

    // Drives a returned pointer through the full ownership contract:
    // read it as a UTF-8 string, parse the JSON, then free it.
    fn take(ptr: *mut c_char) -> serde_json::Value {
        assert!(!ptr.is_null(), "computing fn returned NULL");
        // SAFETY: `ptr` came from a computing fn and is a valid C
        // string; we read it before handing it back to the free fn.
        let text = unsafe { CStr::from_ptr(ptr) }.to_str().expect("valid UTF-8").to_owned();
        // SAFETY: `ptr` is owned by us and freed exactly once here.
        unsafe { rdf_shapes_string_free(ptr) };
        serde_json::from_str(&text).expect("envelope is JSON")
    }

    fn cstr(s: &str) -> CString {
        CString::new(s).expect("no interior NUL")
    }

    #[test]
    fn query_round_trip_returns_ok_envelope() {
        let graph = cstr(GRAPH);
        let sparql = cstr(TX_COUNT);
        // SAFETY: both pointers are valid NUL-terminated C strings.
        let out = unsafe { rdf_shapes_query(graph.as_ptr(), sparql.as_ptr()) };
        let env = take(out);
        let ok = env.get("ok").expect("ok envelope");
        assert_eq!(ok["kind"], "solutions");
        let bindings = ok["json"]["results"]["bindings"].as_array().expect("bindings");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0]["transactions"]["value"], "3");
    }

    #[test]
    fn validate_conforming_round_trip() {
        let data = cstr(CONFORMING);
        let shapes = cstr(SHAPES);
        // SAFETY: both pointers are valid NUL-terminated C strings.
        let out = unsafe { rdf_shapes_validate(data.as_ptr(), shapes.as_ptr()) };
        let env = take(out);
        assert_eq!(env["ok"]["conforms"], true);
    }

    #[test]
    fn validate_violating_round_trip() {
        let data = cstr(VIOLATING);
        let shapes = cstr(SHAPES);
        // SAFETY: both pointers are valid NUL-terminated C strings.
        let out = unsafe { rdf_shapes_validate(data.as_ptr(), shapes.as_ptr()) };
        let env = take(out);
        assert_eq!(env["ok"]["conforms"], false);
        let violations = env["ok"]["violations"].as_array().expect("violations");
        assert!(!violations.is_empty(), "expected at least one violation");
    }

    #[test]
    fn malformed_query_returns_error_envelope() {
        let graph = cstr(GRAPH);
        let sparql = cstr("SELEKT bogus {");
        // SAFETY: both pointers are valid NUL-terminated C strings.
        let out = unsafe { rdf_shapes_query(graph.as_ptr(), sparql.as_ptr()) };
        let env = take(out);
        let message = env["error"].as_str().expect("error envelope");
        assert!(message.contains("query error"), "unexpected message: {message}");
    }

    #[test]
    fn null_argument_returns_error_envelope_not_ub() {
        let sparql = cstr(TX_COUNT);
        // SAFETY: NULL graph is part of the contract; non-NULL sparql.
        let out = unsafe { rdf_shapes_query(std::ptr::null(), sparql.as_ptr()) };
        let env = take(out);
        assert_eq!(env["error"], "graph argument was NULL");
    }

    #[test]
    fn version_round_trip_returns_ok_envelope() {
        let out = rdf_shapes_version();
        let env = take(out);
        assert_eq!(env["ok"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn free_null_is_a_no_op() {
        // SAFETY: NULL is explicitly a no-op per the contract.
        unsafe { rdf_shapes_string_free(std::ptr::null_mut()) };
    }
}
