//! The single error type for the evaluation façade.
//!
//! Every fallible boundary in [`crate::query`] and [`crate::validate`]
//! funnels into [`ShapesError`]. The thin shells map it to a thrown
//! `JsValue` (wasm) or a non-zero exit + stderr (CLI); no failure path
//! panics.

use thiserror::Error;

/// A structured, actionable error from the SPARQL/SHACL façade.
///
/// Variants distinguish *where* a request failed so a consumer can act
/// on it: bad input Turtle/query/shapes ([`ShapesError::Parse`]), a
/// SPARQL evaluation failure ([`ShapesError::Query`]), a SHACL
/// validation failure ([`ShapesError::Validation`]), or a feature the
/// engine deliberately does not support ([`ShapesError::Unsupported`]).
#[derive(Debug, Error)]
pub enum ShapesError {
    /// Malformed input: the graph, query, or shapes failed to parse, or
    /// a result could not be serialized.
    #[error("parse error: {0}")]
    Parse(String),

    /// SPARQL query evaluation failed (e.g. an undefined function or a
    /// runtime evaluation error).
    #[error("query error: {0}")]
    Query(String),

    /// SHACL validation failed to run (e.g. the shapes graph could not
    /// be compiled, or the validator errored).
    #[error("validation error: {0}")]
    Validation(String),

    /// A requested capability is outside this engine's scope, reported
    /// explicitly rather than silently ignored. SHACL-SPARQL
    /// constraints and remote/imported graphs are the two cases.
    #[error("unsupported: {0}")]
    Unsupported(&'static str),
}
