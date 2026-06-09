//! SPARQL 1.1 query evaluation over an in-memory graph.
//!
//! [`query`] loads a Turtle graph into a fresh Oxigraph in-memory
//! [`Store`], evaluates a SPARQL 1.1 query, and returns typed
//! [`QueryResults`]. SELECT/ASK results are serialized through
//! Oxigraph's native SPARQL 1.1 Query Results JSON serializer, so each
//! binding keeps its term kind and datatype; CONSTRUCT/DESCRIBE graphs
//! are serialized as N-Triples.

use oxigraph::io::{RdfFormat, RdfSerializer};
use oxigraph::sparql::results::{QueryResultsFormat, QueryResultsSerializer};
use oxigraph::sparql::{QueryResults as OxQueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::error::ShapesError;
use crate::results::QueryResults;

/// Loads `graph_ttl` into a fresh in-memory store, evaluates `sparql`,
/// and returns the typed [`QueryResults`].
///
/// SELECT results are serialized via Oxigraph's SPARQL 1.1 Query
/// Results JSON serializer (typed terms, not lexical strings); ASK
/// returns a boolean; CONSTRUCT/DESCRIBE returns the result graph as
/// N-Triples. Evaluation is fully in-memory — no filesystem, no
/// network.
///
/// # Errors
///
/// - [`ShapesError::Parse`] if the Turtle graph fails to load or a
///   result cannot be serialized.
/// - [`ShapesError::Query`] if the SPARQL fails to parse or evaluate.
pub fn query(graph_ttl: &str, sparql: &str) -> Result<QueryResults, ShapesError> {
    let store = Store::new().map_err(|e| ShapesError::Parse(format!("store init: {e}")))?;
    store
        .load_from_slice(RdfFormat::Turtle, graph_ttl.as_bytes())
        .map_err(|e| ShapesError::Parse(format!("turtle load: {e}")))?;

    let results = SparqlEvaluator::new()
        .parse_query(sparql)
        .map_err(|e| ShapesError::Query(format!("parse: {e}")))?
        .on_store(&store)
        .execute()
        .map_err(|e| ShapesError::Query(format!("eval: {e}")))?;

    match results {
        OxQueryResults::Solutions(solutions) => {
            let variables = solutions.variables().to_vec();
            let serializer = QueryResultsSerializer::from_format(QueryResultsFormat::Json);
            let mut writer = serializer
                .serialize_solutions_to_writer(Vec::new(), variables)
                .map_err(|e| ShapesError::Parse(format!("results json: {e}")))?;
            for solution in solutions {
                let solution =
                    solution.map_err(|e| ShapesError::Query(format!("solution: {e}")))?;
                writer
                    .serialize(&solution)
                    .map_err(|e| ShapesError::Parse(format!("results json: {e}")))?;
            }
            let bytes =
                writer.finish().map_err(|e| ShapesError::Parse(format!("results json: {e}")))?;
            let json = serde_json::from_slice(&bytes)
                .map_err(|e| ShapesError::Parse(format!("results json: {e}")))?;
            Ok(QueryResults::Solutions { json })
        }
        OxQueryResults::Boolean(value) => Ok(QueryResults::Boolean { value }),
        OxQueryResults::Graph(triples) => {
            let mut serializer =
                RdfSerializer::from_format(RdfFormat::NTriples).for_writer(Vec::new());
            for triple in triples {
                let triple = triple.map_err(|e| ShapesError::Query(format!("triple: {e}")))?;
                serializer
                    .serialize_triple(&triple)
                    .map_err(|e| ShapesError::Parse(format!("ntriples: {e}")))?;
            }
            let bytes =
                serializer.finish().map_err(|e| ShapesError::Parse(format!("ntriples: {e}")))?;
            let ntriples = String::from_utf8(bytes)
                .map_err(|e| ShapesError::Parse(format!("ntriples utf8: {e}")))?;
            Ok(QueryResults::Graph { ntriples })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::query;
    use crate::error::ShapesError;
    use crate::results::QueryResults;

    // A tiny treasury-shaped graph: three cardano:Transaction nodes
    // plus non-transaction noise that must not be counted.
    const SAMPLE: &str = r#"
@prefix cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#> .

<urn:tx:aaaa> a cardano:Transaction .
<urn:tx:bbbb> a cardano:Transaction .
<urn:tx:cccc> a cardano:Transaction .
<urn:out:1>   a cardano:Output .
"#;

    // The real treasury `tx-count` named query.
    const TX_COUNT: &str = r#"
PREFIX cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#>
SELECT (COUNT(DISTINCT ?tx) AS ?transactions)
WHERE { ?tx a cardano:Transaction . }
"#;

    #[test]
    fn select_returns_typed_solutions() {
        let results = query(SAMPLE, TX_COUNT).expect("query ok");
        let QueryResults::Solutions { json } = results else {
            panic!("expected Solutions, got {results:?}");
        };
        // SPARQL Results JSON: head.vars lists the projected variable.
        let vars = json["head"]["vars"].as_array().expect("head.vars present");
        assert_eq!(vars, &[serde_json::json!("transactions")]);
        // One binding row, the count as a typed xsd:integer literal.
        let bindings = json["results"]["bindings"].as_array().expect("bindings present");
        assert_eq!(bindings.len(), 1);
        let term = &bindings[0]["transactions"];
        assert_eq!(term["type"], "literal");
        assert_eq!(term["value"], "3");
        assert_eq!(term["datatype"], "http://www.w3.org/2001/XMLSchema#integer");
    }

    #[test]
    fn ask_returns_boolean() {
        let ask = r#"
PREFIX cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#>
ASK { ?tx a cardano:Transaction . }
"#;
        let results = query(SAMPLE, ask).expect("query ok");
        let QueryResults::Boolean { value } = results else {
            panic!("expected Boolean, got {results:?}");
        };
        assert!(value);
    }

    #[test]
    fn malformed_query_is_a_query_error() {
        let err = query(SAMPLE, "SELEKT bogus {").expect_err("must fail");
        assert!(matches!(err, ShapesError::Query(_)), "expected Query error, got {err:?}");
    }
}
