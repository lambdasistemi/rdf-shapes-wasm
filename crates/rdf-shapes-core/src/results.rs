//! Structured SPARQL query results.
//!
//! [`QueryResults`] is the typed outcome of [`crate::query`]. Its three
//! variants mirror the SPARQL 1.1 query forms:
//!
//! - [`QueryResults::Solutions`] — SELECT, carrying the SPARQL 1.1
//!   Query Results JSON document (`head`/`results`) produced by
//!   Oxigraph's native serializer, so every binding keeps its term
//!   kind and datatype rather than a lexical string.
//! - [`QueryResults::Boolean`] — ASK, a single boolean.
//! - [`QueryResults::Graph`] — CONSTRUCT/DESCRIBE, the resulting graph
//!   serialized as N-Triples.
//!
//! The enum is `serde`-tagged so a JSON consumer can branch on `kind`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The typed result of a SPARQL 1.1 query.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum QueryResults {
    /// SELECT result: the SPARQL 1.1 Query Results JSON object, with
    /// `head.vars` and `results.bindings` carrying typed terms.
    Solutions {
        /// The Query Results JSON document, verbatim from Oxigraph's
        /// JSON serializer (typed terms, not lexical forms).
        json: Value,
    },
    /// ASK result: a single boolean.
    Boolean {
        /// Whether the ASK pattern matched.
        value: bool,
    },
    /// CONSTRUCT/DESCRIBE result: the graph serialized as N-Triples.
    Graph {
        /// The result graph as N-Triples (one triple per line).
        ntriples: String,
    },
}
