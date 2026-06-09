//! Portable core logic for `rdf-shapes-wasm`.
//!
//! This crate holds all business logic and is compiled both for the
//! native host triple and for `wasm32-unknown-unknown`. It has no
//! dependency on `wasm-bindgen` and performs no I/O, threading,
//! filesystem, or socket access, so it stays portable across every
//! target.
//!
//! SPIKE (#3): this branch additionally wires Oxigraph's in-memory
//! store and exposes [`query`], proving SPARQL 1.1 runs over
//! `wasm32-unknown-unknown`. Throwaway — not the production surface.

use oxigraph::io::RdfFormat;
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;
use serde_json::{Map, Value};

/// Errors raised while loading a graph or evaluating a SPARQL query.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    /// The in-memory store could not be created.
    #[error("store init failed: {0}")]
    Store(String),
    /// The Turtle payload failed to parse / load.
    #[error("turtle load failed: {0}")]
    Load(String),
    /// The SPARQL text failed to parse.
    #[error("sparql parse failed: {0}")]
    Parse(String),
    /// Query evaluation failed.
    #[error("sparql eval failed: {0}")]
    Eval(String),
    /// The result rows could not be serialized to JSON.
    #[error("serialize failed: {0}")]
    Serialize(String),
}

/// Loads `turtle_data` into a fresh in-memory store, runs `sparql`,
/// and returns the result as a JSON string.
///
/// For a SELECT query the result is a JSON array of row objects, each
/// mapping bound variable name to its term's lexical form (the
/// `Display` rendering used in SPARQL results). For ASK it is a single
/// `{"boolean": true|false}` object.
///
/// This is the spike surface that retires the "does Oxigraph run
/// SPARQL 1.1 on wasm32?" risk — backed entirely by the in-memory
/// store, no native storage backend.
///
/// # Errors
///
/// Returns a [`QueryError`] if the store cannot be created, the Turtle
/// fails to load, the SPARQL fails to parse or evaluate, or the rows
/// cannot be serialized.
pub fn query(turtle_data: &str, sparql: &str) -> Result<String, QueryError> {
    let store =
        Store::new().map_err(|e| QueryError::Store(e.to_string()))?;
    store
        .load_from_slice(RdfFormat::Turtle, turtle_data.as_bytes())
        .map_err(|e| QueryError::Load(e.to_string()))?;

    let results = SparqlEvaluator::new()
        .parse_query(sparql)
        .map_err(|e| QueryError::Parse(e.to_string()))?
        .on_store(&store)
        .execute()
        .map_err(|e| QueryError::Eval(e.to_string()))?;

    let value = match results {
        QueryResults::Solutions(solutions) => {
            let mut rows: Vec<Value> = Vec::new();
            for solution in solutions {
                let solution = solution
                    .map_err(|e| QueryError::Eval(e.to_string()))?;
                let mut row = Map::new();
                for (var, term) in solution.iter() {
                    row.insert(
                        var.as_str().to_owned(),
                        Value::String(term.to_string()),
                    );
                }
                rows.push(Value::Object(row));
            }
            Value::Array(rows)
        }
        QueryResults::Boolean(b) => {
            let mut row = Map::new();
            row.insert("boolean".to_owned(), Value::Bool(b));
            Value::Object(row)
        }
        QueryResults::Graph(_) => {
            return Err(QueryError::Eval(
                "CONSTRUCT/DESCRIBE not supported by this spike"
                    .to_owned(),
            ));
        }
    };

    serde_json::to_string(&value)
        .map_err(|e| QueryError::Serialize(e.to_string()))
}

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
    use super::{ping, query, version};

    #[test]
    fn version_matches_cargo_metadata() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn ping_prefixes_with_pong() {
        assert_eq!(ping("hello"), "pong: hello");
    }

    // SPIKE (#3): the treasury `tx-count` named query over a tiny
    // hand-crafted graph with three cardano:Transaction nodes.
    const SAMPLE_TURTLE: &str = r#"
@prefix cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#> .

<urn:tx:1> a cardano:Transaction .
<urn:tx:2> a cardano:Transaction .
<urn:tx:3> a cardano:Transaction .
"#;

    const TX_COUNT: &str = r#"
PREFIX cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#>

SELECT (COUNT(DISTINCT ?tx) AS ?transactions)
WHERE {
  ?tx a cardano:Transaction .
}
"#;

    #[test]
    fn tx_count_returns_three() {
        let json = query(SAMPLE_TURTLE, TX_COUNT).expect("query ok");
        let rows: serde_json::Value =
            serde_json::from_str(&json).expect("valid json");
        let count = rows[0]["transactions"]
            .as_str()
            .expect("transactions bound");
        // SPARQL renders the aggregate as a typed integer literal.
        assert!(count.starts_with("\"3\"") || count == "3");
    }
}
