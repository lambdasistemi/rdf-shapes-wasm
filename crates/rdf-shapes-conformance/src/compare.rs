use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Context};
use oxrdf::{NamedOrBlankNode, Term};
use oxrdfio::{RdfFormat, RdfParser};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SelectMode {
    Multiset,
    Ordered,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ComparableQuery {
    Select { json: Value, mode: SelectMode },
    Ask(bool),
    Graph { ntriples: String },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ComparableShacl {
    pub conforms: bool,
    pub violations: Vec<ViolationKey>,
}

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct ViolationKey {
    pub focus_node: String,
    pub source_constraint_component: String,
    pub result_path: Option<String>,
}

pub fn query_equal(left: &ComparableQuery, right: &ComparableQuery) -> anyhow::Result<bool> {
    match (left, right) {
        (
            ComparableQuery::Select { json: left, mode: left_mode },
            ComparableQuery::Select { json: right, mode: right_mode },
        ) if left_mode == right_mode => select_equal(left, right, *left_mode),
        (ComparableQuery::Ask(left), ComparableQuery::Ask(right)) => Ok(left == right),
        (ComparableQuery::Graph { ntriples: left }, ComparableQuery::Graph { ntriples: right }) => {
            graph_equal(left, right)
        }
        _ => Ok(false),
    }
}

pub fn shacl_equal(left: &ComparableShacl, right: &ComparableShacl) -> bool {
    if left.conforms != right.conforms {
        return false;
    }
    let mut left_violations = left.violations.clone();
    let mut right_violations = right.violations.clone();
    left_violations.sort();
    right_violations.sort();
    left_violations == right_violations
}

fn select_equal(left: &Value, right: &Value, mode: SelectMode) -> anyhow::Result<bool> {
    let left_vars = select_vars(left)?;
    let right_vars = select_vars(right)?;
    if left_vars != right_vars {
        return Ok(false);
    }

    let mut left_rows = select_rows(left, &left_vars)?;
    let mut right_rows = select_rows(right, &right_vars)?;
    if mode == SelectMode::Multiset {
        left_rows.sort();
        right_rows.sort();
    }
    Ok(left_rows == right_rows)
}

fn select_vars(json: &Value) -> anyhow::Result<Vec<String>> {
    let vars = json
        .pointer("/head/vars")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("SPARQL Results JSON missing head.vars"))?;
    vars.iter()
        .map(|var| {
            var.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| anyhow!("SPARQL Results JSON variable is not a string"))
        })
        .collect()
}

fn select_rows(json: &Value, vars: &[String]) -> anyhow::Result<Vec<String>> {
    let bindings = json
        .pointer("/results/bindings")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("SPARQL Results JSON missing results.bindings"))?;
    bindings
        .iter()
        .map(|row| {
            let mut blank_nodes = BTreeMap::new();
            let fields = vars
                .iter()
                .map(|var| {
                    let value = row.get(var);
                    normalized_binding(value, &mut blank_nodes)
                        .with_context(|| format!("normalizing binding {var}"))
                        .map(|term| format!("{var}={term}"))
                })
                .collect::<anyhow::Result<Vec<_>>>()?;
            Ok(fields.join("\u{1f}"))
        })
        .collect()
}

fn normalized_binding(
    value: Option<&Value>,
    blank_nodes: &mut BTreeMap<String, String>,
) -> anyhow::Result<String> {
    let Some(value) = value else {
        return Ok("unbound".to_owned());
    };
    let object = value.as_object().ok_or_else(|| anyhow!("binding term is not an object"))?;
    let kind = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("binding term missing type"))?;
    let lexical = object
        .get("value")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("binding term missing value"))?;
    match kind {
        "bnode" => {
            let next = blank_nodes.len();
            let canonical =
                blank_nodes.entry(lexical.to_owned()).or_insert_with(|| format!("_:b{next}"));
            Ok(format!("bnode:{canonical}"))
        }
        "literal" => {
            let datatype = object
                .get("datatype")
                .and_then(Value::as_str)
                .unwrap_or("http://www.w3.org/2001/XMLSchema#string");
            let lang = object.get("xml:lang").and_then(Value::as_str).unwrap_or("");
            Ok(format!("literal:{lexical}\u{1e}{datatype}\u{1e}{lang}"))
        }
        "uri" => Ok(format!("uri:{lexical}")),
        other => Ok(format!("{other}:{lexical}")),
    }
}

fn graph_equal(left: &str, right: &str) -> anyhow::Result<bool> {
    let left = graph_terms(left)?;
    let right = graph_terms(right)?;
    if left.blank_nodes.len() != right.blank_nodes.len() {
        return Ok(false);
    }
    if left.blank_nodes.is_empty() {
        return Ok(left.render(&BTreeMap::new()) == right.render(&BTreeMap::new()));
    }
    if left.blank_nodes.len() > 8 {
        return Ok(canonical_triples(left.triples)? == canonical_triples(right.triples)?);
    }

    let left_ids = left.blank_nodes.iter().cloned().collect::<Vec<_>>();
    let right_ids = right.blank_nodes.iter().cloned().collect::<Vec<_>>();
    let left_rendered = left.render(&BTreeMap::new());
    for permutation in permutations(left_ids) {
        let mapping = right_ids.iter().cloned().zip(permutation).collect::<BTreeMap<_, _>>();
        if left_rendered == right.render(&mapping) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Debug)]
struct ParsedGraph<'a> {
    triples: &'a str,
    rows: Vec<(GraphNode, String, GraphNode)>,
    blank_nodes: BTreeSet<String>,
}

impl ParsedGraph<'_> {
    fn render(&self, mapping: &BTreeMap<String, String>) -> Vec<String> {
        let mut rows = self
            .rows
            .iter()
            .map(|(subject, predicate, object)| {
                format!("{} {predicate} {}", subject.render(mapping), object.render(mapping))
            })
            .collect::<Vec<_>>();
        rows.sort();
        rows
    }
}

#[derive(Debug)]
enum GraphNode {
    Stable(String),
    Blank(String),
}

impl GraphNode {
    fn render(&self, mapping: &BTreeMap<String, String>) -> String {
        match self {
            Self::Stable(value) => value.clone(),
            Self::Blank(id) => {
                format!("_:{}", mapping.get(id).map(String::as_str).unwrap_or(id.as_str()))
            }
        }
    }
}

fn graph_terms(ntriples: &str) -> anyhow::Result<ParsedGraph<'_>> {
    let triples = parse_ntriples(ntriples)?;
    let mut blank_nodes = BTreeSet::new();
    let rows = triples
        .into_iter()
        .map(|(subject, predicate, object)| {
            let subject = graph_subject(subject, &mut blank_nodes);
            let predicate = format!("<{}>", predicate.as_str());
            let object = graph_term(object, &mut blank_nodes);
            (subject, predicate, object)
        })
        .collect();
    Ok(ParsedGraph { triples: ntriples, rows, blank_nodes })
}

fn parse_ntriples(
    ntriples: &str,
) -> anyhow::Result<Vec<(NamedOrBlankNode, oxrdf::NamedNode, Term)>> {
    parse_rdf_graph(ntriples, RdfFormat::NTriples)
        .or_else(|_| parse_rdf_graph(ntriples, RdfFormat::Turtle))
}

fn parse_rdf_graph(
    graph: &str,
    format: RdfFormat,
) -> anyhow::Result<Vec<(NamedOrBlankNode, oxrdf::NamedNode, Term)>> {
    RdfParser::from_format(format)
        .without_named_graphs()
        .for_slice(graph)
        .map(|quad| {
            let quad = quad.context("parsing RDF graph")?;
            Ok((quad.subject, quad.predicate, quad.object))
        })
        .collect::<anyhow::Result<Vec<_>>>()
}

fn graph_subject(subject: NamedOrBlankNode, blank_nodes: &mut BTreeSet<String>) -> GraphNode {
    match subject {
        NamedOrBlankNode::NamedNode(node) => GraphNode::Stable(format!("<{}>", node.as_str())),
        NamedOrBlankNode::BlankNode(node) => {
            let id = node.into_string();
            blank_nodes.insert(id.clone());
            GraphNode::Blank(id)
        }
    }
}

fn graph_term(term: Term, blank_nodes: &mut BTreeSet<String>) -> GraphNode {
    match term {
        Term::NamedNode(node) => GraphNode::Stable(format!("<{}>", node.as_str())),
        Term::BlankNode(node) => {
            let id = node.into_string();
            blank_nodes.insert(id.clone());
            GraphNode::Blank(id)
        }
        Term::Literal(literal) => GraphNode::Stable(literal.to_string()),
        Term::Triple(triple) => GraphNode::Stable(format!("<<{}>>", triple)),
    }
}

fn permutations(values: Vec<String>) -> Vec<Vec<String>> {
    if values.len() <= 1 {
        return vec![values];
    }
    let mut out = Vec::new();
    for index in 0..values.len() {
        let mut rest = values.clone();
        let head = rest.remove(index);
        for mut tail in permutations(rest) {
            let mut value = vec![head.clone()];
            value.append(&mut tail);
            out.push(value);
        }
    }
    out
}

fn canonical_triples(ntriples: &str) -> anyhow::Result<Vec<String>> {
    let triples = parse_ntriples(ntriples)?;

    let mut blank_nodes = BTreeMap::new();
    let mut rows = triples
        .into_iter()
        .map(|(subject, predicate, object)| {
            let subject = normalized_subject(&subject, &mut blank_nodes);
            let predicate = format!("<{}>", predicate.as_str());
            let object = normalized_term(&object, &mut blank_nodes)?;
            Ok(format!("{subject} {predicate} {object}"))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    rows.sort();
    Ok(rows)
}

fn normalized_subject(
    subject: &NamedOrBlankNode,
    blank_nodes: &mut BTreeMap<String, String>,
) -> String {
    match subject {
        NamedOrBlankNode::NamedNode(node) => format!("<{}>", node.as_str()),
        NamedOrBlankNode::BlankNode(node) => normalized_blank_node(node.as_str(), blank_nodes),
    }
}

fn normalized_term(
    term: &Term,
    blank_nodes: &mut BTreeMap<String, String>,
) -> anyhow::Result<String> {
    match term {
        Term::NamedNode(node) => Ok(format!("<{}>", node.as_str())),
        Term::BlankNode(node) => Ok(normalized_blank_node(node.as_str(), blank_nodes)),
        Term::Literal(literal) => Ok(literal.to_string()),
        Term::Triple(triple) => Ok(format!("<<{}>>", triple)),
    }
}

fn normalized_blank_node(id: &str, blank_nodes: &mut BTreeMap<String, String>) -> String {
    let next = blank_nodes.len();
    blank_nodes.entry(id.to_owned()).or_insert_with(|| format!("_:b{next}")).clone()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        query_equal, shacl_equal, ComparableQuery, ComparableShacl, SelectMode, ViolationKey,
    };

    #[test]
    fn select_multiset_ignores_row_order() {
        let left = ComparableQuery::Select {
            mode: SelectMode::Multiset,
            json: json!({
                "head": { "vars": ["name"] },
                "results": { "bindings": [
                    { "name": { "type": "literal", "value": "Ada" } },
                    { "name": { "type": "literal", "value": "Babbage" } }
                ] }
            }),
        };
        let right = ComparableQuery::Select {
            mode: SelectMode::Multiset,
            json: json!({
                "head": { "vars": ["name"] },
                "results": { "bindings": [
                    { "name": { "type": "literal", "value": "Babbage" } },
                    { "name": { "type": "literal", "value": "Ada" } }
                ] }
            }),
        };

        assert!(query_equal(&left, &right).expect("comparison succeeds"));
    }

    #[test]
    fn select_ordered_respects_row_order() {
        let left = ComparableQuery::Select {
            mode: SelectMode::Ordered,
            json: json!({
                "head": { "vars": ["name"] },
                "results": { "bindings": [
                    { "name": { "type": "literal", "value": "Ada" } },
                    { "name": { "type": "literal", "value": "Babbage" } }
                ] }
            }),
        };
        let right = ComparableQuery::Select {
            mode: SelectMode::Ordered,
            json: json!({
                "head": { "vars": ["name"] },
                "results": { "bindings": [
                    { "name": { "type": "literal", "value": "Babbage" } },
                    { "name": { "type": "literal", "value": "Ada" } }
                ] }
            }),
        };

        assert!(!query_equal(&left, &right).expect("comparison succeeds"));
    }

    #[test]
    fn select_canonicalizes_blank_node_labels() {
        let left = ComparableQuery::Select {
            mode: SelectMode::Multiset,
            json: json!({
                "head": { "vars": ["node", "label"] },
                "results": { "bindings": [
                    {
                        "node": { "type": "bnode", "value": "left1" },
                        "label": { "type": "literal", "value": "same" }
                    }
                ] }
            }),
        };
        let right = ComparableQuery::Select {
            mode: SelectMode::Multiset,
            json: json!({
                "head": { "vars": ["node", "label"] },
                "results": { "bindings": [
                    {
                        "node": { "type": "bnode", "value": "right9" },
                        "label": { "type": "literal", "value": "same" }
                    }
                ] }
            }),
        };

        assert!(query_equal(&left, &right).expect("comparison succeeds"));
    }

    #[test]
    fn graph_canonicalizes_blank_node_labels() {
        let left =
            ComparableQuery::Graph { ntriples: "_:a <http://example/p> \"v\" .\n".to_owned() };
        let right =
            ComparableQuery::Graph { ntriples: "_:z <http://example/p> \"v\" .\n".to_owned() };

        assert!(query_equal(&left, &right).expect("comparison succeeds"));
    }

    #[test]
    fn shacl_compares_violation_identity_ignoring_order_and_messages() {
        let left = ComparableShacl {
            conforms: false,
            violations: vec![
                ViolationKey {
                    focus_node: "urn:bad".to_owned(),
                    source_constraint_component:
                        "http://www.w3.org/ns/shacl#MinCountConstraintComponent".to_owned(),
                    result_path: Some("urn:path".to_owned()),
                },
                ViolationKey {
                    focus_node: "urn:bad".to_owned(),
                    source_constraint_component:
                        "http://www.w3.org/ns/shacl#DatatypeConstraintComponent".to_owned(),
                    result_path: Some("urn:slot".to_owned()),
                },
            ],
        };
        let right = ComparableShacl {
            conforms: false,
            violations: vec![
                ViolationKey {
                    focus_node: "urn:bad".to_owned(),
                    source_constraint_component:
                        "http://www.w3.org/ns/shacl#DatatypeConstraintComponent".to_owned(),
                    result_path: Some("urn:slot".to_owned()),
                },
                ViolationKey {
                    focus_node: "urn:bad".to_owned(),
                    source_constraint_component:
                        "http://www.w3.org/ns/shacl#MinCountConstraintComponent".to_owned(),
                    result_path: Some("urn:path".to_owned()),
                },
            ],
        };

        assert!(shacl_equal(&left, &right));
    }
}
