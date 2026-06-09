//! THROWAWAY de-risking spike (issue #4): SHACL Core validation in wasm.
//!
//! Proves that the rudof project's [`shacl_validation`] crate compiles
//! and runs for `wasm32-unknown-unknown` over **in-memory Turtle**, with
//! no filesystem, no network, and no SPARQL endpoint — i.e. SHACL Core
//! only, via the Native engine.
//!
//! The single exported entry point is [`validate`]:
//! `validate(data_ttl, shapes_ttl) -> json report`.
//!
//! This is reference code for the epic's decision gate. It deliberately
//! does NOT follow the production pure-core / thin-shell split or the
//! clippy/fmt/deny lint surface — spikes are exempt.

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use rudof_rdf::rdf_core::RDFFormat;
use rudof_rdf::rdf_impl::{InMemoryGraph, ReaderMode};
use shacl_ir::compiled::schema_ir::SchemaIR;
use shacl_validation::shacl_processor::{
    GraphValidation, ShaclProcessor, ShaclValidationMode,
};
use shacl_validation::store::Graph;

/// One SHACL violation, flattened to displayable strings for JSON.
#[derive(Serialize, Debug)]
pub struct Violation {
    /// The focus node the constraint was evaluated against.
    pub focus_node: String,
    /// The `sh:path` of the failing property shape, if any.
    pub path: Option<String>,
    /// The offending value node, if the result carries one.
    pub value: Option<String>,
    /// The `sh:sourceConstraintComponent` IRI (e.g. MinCount, Datatype).
    pub component: String,
    /// `sh:resultSeverity`, rendered (e.g. `Violation`).
    pub severity: String,
    /// Optional human-readable message.
    pub message: Option<String>,
}

/// A conformance report: the boolean flag plus any violations.
#[derive(Serialize, Debug)]
pub struct Report {
    /// `true` iff the data graph conforms to the shapes graph.
    pub conforms: bool,
    /// Number of violation-severity results.
    pub violation_count: usize,
    /// The flattened results.
    pub violations: Vec<Violation>,
    /// Set when validation itself failed (parse / compile error).
    pub error: Option<String>,
}

/// Runs SHACL Core validation of `data_ttl` against `shapes_ttl`.
///
/// Both inputs are Turtle source strings held entirely in memory. The
/// rudof Native engine is used (the `sparql` feature is compiled out),
/// so only SHACL Core constraints are evaluated. Returns a [`Report`].
///
/// This is the portable core; the wasm shim [`validate`] just JSON-
/// encodes its output. Kept native-testable so the logic is exercised
/// off-wasm in `nix flake check`-style runs.
#[must_use]
pub fn validate_core(data_ttl: &str, shapes_ttl: &str) -> Report {
    match run(data_ttl, shapes_ttl) {
        Ok(report) => report,
        Err(e) => Report {
            conforms: false,
            violation_count: 0,
            violations: Vec::new(),
            error: Some(e),
        },
    }
}

fn run(data_ttl: &str, shapes_ttl: &str) -> Result<Report, String> {
    let reader_mode = ReaderMode::default();

    // 1. Parse the data graph from a Turtle string (no fs).
    let data_graph =
        InMemoryGraph::from_str(data_ttl, &RDFFormat::Turtle, None, &reader_mode)
            .map_err(|e| format!("data parse error: {e}"))?;

    // 2. Compile the shapes graph (Turtle string) into the SHACL IR.
    let schema_ir =
        SchemaIR::from_str(shapes_ttl, &RDFFormat::Turtle, None, &reader_mode)
            .map_err(|e| format!("shapes compile error: {e}"))?;

    // 3. Wrap the data graph in the in-memory store + Native processor.
    let store = Graph::from_graph(data_graph)
        .map_err(|e| format!("store build error: {e}"))?;
    let mut processor =
        GraphValidation::from_graph(store, ShaclValidationMode::Native);

    // 4. Validate; map the report into our serializable shape.
    let report = processor
        .validate(&schema_ir)
        .map_err(|e| format!("validation error: {e}"))?;

    let violations = report
        .results()
        .iter()
        .map(|r| Violation {
            focus_node: r.focus_node().to_string(),
            path: r.path().map(std::string::ToString::to_string),
            value: r.value().map(std::string::ToString::to_string),
            component: r.component().to_string(),
            severity: format!("{:?}", r.severity()),
            message: r.message().map(std::borrow::ToOwned::to_owned),
        })
        .collect();

    Ok(Report {
        conforms: report.conforms(),
        violation_count: report.count_violations(),
        violations,
        error: None,
    })
}

/// Installs a panic hook forwarding Rust panics to `console.error`.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// wasm entry point: validate `data_ttl` against `shapes_ttl`, returning
/// the [`Report`] as a JSON string.
#[wasm_bindgen]
#[must_use]
pub fn validate(data_ttl: &str, shapes_ttl: &str) -> String {
    let report = validate_core(data_ttl, shapes_ttl);
    serde_json::to_string(&report)
        .unwrap_or_else(|e| format!(r#"{{"error":"json encode: {e}"}}"#))
}

#[cfg(test)]
mod tests {
    use super::validate_core;

    const SHAPES: &str = include_str!("../testdata/history-entry.shacl.ttl");
    const CONFORMING: &str = include_str!("../testdata/conforming.ttl");
    const VIOLATING: &str = include_str!("../testdata/violating.ttl");

    #[test]
    fn conforming_graph_conforms() {
        let report = validate_core(CONFORMING, SHAPES);
        assert!(report.error.is_none(), "unexpected error: {:?}", report.error);
        assert!(report.conforms, "expected conformance, got {:?}", report.violations);
    }

    #[test]
    fn violating_graph_reports_violation() {
        let report = validate_core(VIOLATING, SHAPES);
        assert!(report.error.is_none(), "unexpected error: {:?}", report.error);
        assert!(!report.conforms, "expected non-conformance");
        assert!(report.violation_count >= 1, "expected at least one violation");
    }
}
