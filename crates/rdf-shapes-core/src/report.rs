//! Structured SHACL validation report.
//!
//! [`ValidationReport`] is the typed outcome of [`crate::validate`]: an
//! overall [`conforms`](ValidationReport::conforms) flag plus one
//! [`Violation`] per failing result, mapped from rudof's native
//! validation report.

use serde::{Deserialize, Serialize};

/// A SHACL conformance report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// `true` iff the data graph conforms to the shapes graph (no
    /// violation-severity results).
    pub conforms: bool,
    /// One entry per validation result, empty when `conforms` is true.
    pub violations: Vec<Violation>,
}

/// A single SHACL validation result, flattened to displayable strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// The focus node the constraint was evaluated against.
    pub focus_node: String,
    /// The `sh:resultPath` of the failing property shape, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// The offending value node, when the result carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// The IRI identifying the failed constraint. rudof reports the
    /// SHACL constraint *parameter* IRI (e.g. `sh:minCount`,
    /// `sh:datatype`, `sh:nodeKind`) rather than the formal
    /// `sh:...ConstraintComponent` IRI.
    pub source_constraint_component: String,
    /// The human-readable `sh:resultMessage`, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// The `sh:resultSeverity`, rendered (e.g. `Violation`).
    pub severity: String,
}
