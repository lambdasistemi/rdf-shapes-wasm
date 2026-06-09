//! SHACL Core validation over in-memory graphs.
//!
//! [`validate`] parses a data graph and a SHACL shapes graph from
//! Turtle, runs rudof's Native (in-memory) validation engine — SHACL
//! **Core** only — and maps the result into a structured
//! [`ValidationReport`]. Constraints outside SHACL Core (SHACL-SPARQL)
//! and remote/imported graphs are reported as
//! [`ShapesError::Unsupported`], never silently ignored.

use rudof_rdf::rdf_core::RDFFormat;
use rudof_rdf::rdf_impl::{InMemoryGraph, ReaderMode};
use shacl_ir::compiled::schema_ir::SchemaIR;
use shacl_validation::shacl_processor::{
    GraphValidation, ShaclProcessor, ShaclValidationMode,
};
use shacl_validation::store::Graph;

use crate::error::ShapesError;
use crate::report::{ValidationReport, Violation};

/// Validates `data_ttl` against `shapes_ttl` (SHACL **Core**).
///
/// Both inputs are in-memory Turtle strings. rudof's Native engine is
/// used (the `sparql` feature is compiled out), so only SHACL Core
/// constraints are evaluated; SHACL-SPARQL and remote/imported graphs
/// are out of scope and surface as [`ShapesError::Unsupported`] rather
/// than a silent pass.
///
/// # Errors
///
/// - [`ShapesError::Parse`] if the data graph or shapes graph fails to
///   parse / compile.
/// - [`ShapesError::Validation`] if the validator itself errors.
/// - [`ShapesError::Unsupported`] if the shapes require a capability
///   outside SHACL Core (SHACL-SPARQL, remote graphs).
pub fn validate(
    data_ttl: &str,
    shapes_ttl: &str,
) -> Result<ValidationReport, ShapesError> {
    let reader_mode = ReaderMode::default();

    // Parse the data graph from Turtle (no fs, no network).
    let data_graph = InMemoryGraph::from_str(
        data_ttl,
        &RDFFormat::Turtle,
        None,
        &reader_mode,
    )
    .map_err(|e| ShapesError::Parse(format!("data graph: {e}")))?;

    // Compile the shapes graph into the SHACL IR.
    let schema_ir = SchemaIR::from_str(
        shapes_ttl,
        &RDFFormat::Turtle,
        None,
        &reader_mode,
    )
    .map_err(|e| classify_shapes_error(&e.to_string()))?;

    // Wrap the data graph in the in-memory store + Native processor.
    let store = Graph::from_graph(data_graph)
        .map_err(|e| ShapesError::Validation(format!("store: {e}")))?;
    let mut processor =
        GraphValidation::from_graph(store, ShaclValidationMode::Native);

    let report = processor
        .validate(&schema_ir)
        .map_err(|e| classify_validation_error(&e.to_string()))?;

    let mut violations: Vec<Violation> = report
        .results()
        .iter()
        .map(|r| Violation {
            focus_node: r.focus_node().to_string(),
            path: r.path().map(std::string::ToString::to_string),
            value: r.value().map(std::string::ToString::to_string),
            source_constraint_component: r.component().to_string(),
            message: r.message().map(std::borrow::ToOwned::to_owned),
            severity: format!("{:?}", r.severity()),
        })
        .collect();

    // rudof does not guarantee result ordering; sort on a stable key
    // so the report is deterministic (and so the wasm and CLI surfaces
    // produce byte-identical output for the same inputs).
    violations.sort_by(|a, b| {
        (
            &a.focus_node,
            &a.source_constraint_component,
            &a.path,
            &a.value,
        )
            .cmp(&(
                &b.focus_node,
                &b.source_constraint_component,
                &b.path,
                &b.value,
            ))
    });

    Ok(ValidationReport {
        conforms: report.conforms(),
        violations,
    })
}

/// Routes a shapes-compilation error to [`ShapesError::Unsupported`]
/// when it names a SHACL-SPARQL / remote-graph construct, otherwise to
/// [`ShapesError::Parse`].
fn classify_shapes_error(message: &str) -> ShapesError {
    if mentions_unsupported(message) {
        ShapesError::Unsupported(
            "SHACL-SPARQL constraints and remote graphs are not supported",
        )
    } else {
        ShapesError::Parse(format!("shapes graph: {message}"))
    }
}

/// Routes a validation-run error to [`ShapesError::Unsupported`] when
/// it names a SHACL-SPARQL / remote-graph construct, otherwise to
/// [`ShapesError::Validation`].
fn classify_validation_error(message: &str) -> ShapesError {
    if mentions_unsupported(message) {
        ShapesError::Unsupported(
            "SHACL-SPARQL constraints and remote graphs are not supported",
        )
    } else {
        ShapesError::Validation(message.to_owned())
    }
}

/// Heuristic: does the engine error reference an out-of-scope feature?
fn mentions_unsupported(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("sparql")
        || lower.contains("sh:sparql")
        || lower.contains("remote")
}

#[cfg(test)]
mod tests {
    use super::validate;

    const SHAPES: &str = include_str!("../testdata/history-entry.shacl.ttl");
    const CONFORMING: &str = include_str!("../testdata/conforming.ttl");
    const VIOLATING: &str = include_str!("../testdata/violating.ttl");

    #[test]
    fn conforming_graph_conforms() {
        let report = validate(CONFORMING, SHAPES).expect("validate ok");
        assert!(
            report.conforms,
            "expected conformance, got {:?}",
            report.violations
        );
        assert!(report.violations.is_empty());
    }

    #[test]
    fn violating_graph_reports_each_violation() {
        let report = validate(VIOLATING, SHAPES).expect("validate ok");
        assert!(!report.conforms, "expected non-conformance");

        // The violating graph trips exactly three SHACL Core
        // constraint kinds: sh:minCount (missing atx:direction),
        // sh:datatype (atx:slot a string, not xsd:integer), and
        // sh:nodeKind (atx:tx a literal, not an IRI). Each must show
        // up with its source constraint component. rudof reports the
        // SHACL constraint *parameter* IRI (e.g. sh:minCount), so the
        // match is case-insensitive on that identity.
        let components: Vec<String> = report
            .violations
            .iter()
            .map(|v| v.source_constraint_component.to_ascii_lowercase())
            .collect();
        assert!(
            components.iter().any(|c| c.contains("mincount")),
            "expected a minCount violation, got {components:?}"
        );
        assert!(
            components.iter().any(|c| c.contains("datatype")),
            "expected a datatype violation, got {components:?}"
        );
        assert!(
            components.iter().any(|c| c.contains("nodekind")),
            "expected a nodeKind violation, got {components:?}"
        );

        // Every violation names its focus node and a severity.
        for v in &report.violations {
            assert!(!v.focus_node.is_empty());
            assert!(!v.severity.is_empty());
        }
    }
}
