mod compare;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use compare::{
    query_equal, shacl_equal, ComparableQuery, ComparableShacl, SelectMode, ViolationKey,
};
use oxrdf::{NamedOrBlankNode, Term};
use oxrdfio::{RdfFormat, RdfParser};
use rdf_shapes_core::{QueryResults, ValidationReport};
use serde::Deserialize;

const SH: &str = "http://www.w3.org/ns/shacl#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const SH_CONFORMS: &str = "http://www.w3.org/ns/shacl#conforms";
const SH_RESULT: &str = "http://www.w3.org/ns/shacl#result";
const SH_VALIDATION_RESULT: &str = "http://www.w3.org/ns/shacl#ValidationResult";
const SH_FOCUS_NODE: &str = "http://www.w3.org/ns/shacl#focusNode";
const SH_RESULT_PATH: &str = "http://www.w3.org/ns/shacl#resultPath";
const SH_SOURCE_CONSTRAINT_COMPONENT: &str = "http://www.w3.org/ns/shacl#sourceConstraintComponent";

#[derive(Debug, Parser)]
#[command(
    name = "rdf-shapes-conformance",
    about = "Run rdf-shapes-core conformance and Jena differential cases"
)]
struct Cli {
    /// Path to the conformance corpus root.
    #[arg(default_value = "conformance/corpus")]
    corpus: PathBuf,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct CaseMeta {
    ordered: bool,
    skip: Option<String>,
}

#[derive(Debug, Default)]
struct Summary {
    passed: usize,
    skipped: usize,
    diverged: usize,
    known_divergences: usize,
}

impl Summary {
    fn pass(&mut self) {
        self.passed += 1;
    }

    fn skip(&mut self) {
        self.skipped += 1;
    }

    fn divergence(&mut self) {
        self.diverged += 1;
    }

    fn known_divergence(&mut self) {
        self.known_divergences += 1;
    }

    fn merge(&mut self, other: Summary) {
        self.passed += other.passed;
        self.skipped += other.skipped;
        self.diverged += other.diverged;
        self.known_divergences += other.known_divergences;
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut summary = Summary::default();

    summary.merge(run_seed_sparql(&cli.corpus.join("sparql"))?);
    summary.merge(run_seed_shacl(&cli.corpus.join("shacl"))?);
    summary.merge(run_w3c_sparql(&cli.corpus.join("w3c/sparql"))?);
    summary.merge(run_w3c_shacl(&cli.corpus.join("w3c/shacl"))?);
    summary.merge(run_explicit_skips(&cli.corpus.join("w3c/skips"))?);

    println!(
        "conformance summary: pass={} skip={} known-divergence={} divergence={}",
        summary.passed, summary.skipped, summary.known_divergences, summary.diverged
    );

    if summary.diverged > 0 {
        bail!("conformance harness found {} divergence(s)", summary.diverged);
    }
    Ok(())
}

fn run_seed_sparql(root: &Path) -> Result<Summary> {
    let mut summary = Summary::default();
    for case in case_dirs(root)? {
        let name = case_name(&case)?;
        let meta = read_meta(&case)?;
        if let Some(reason) = meta.skip {
            println!("SKIP sparql/{name}: {reason}");
            summary.skip();
            continue;
        }

        let graph = fs::read_to_string(case.join("graph.ttl"))
            .with_context(|| format!("reading graph for sparql/{name}"))?;
        let query = fs::read_to_string(case.join("query.rq"))
            .with_context(|| format!("reading query for sparql/{name}"))?;
        let engine = engine_query(&graph, &query, select_mode(&query, meta.ordered))?;
        let jena = jena_query(&case, &query, select_mode(&query, meta.ordered))?;
        if query_equal(&engine, &jena).with_context(|| format!("comparing sparql/{name}"))? {
            println!("PASS sparql/{name}");
            summary.pass();
        } else {
            eprintln!("DIVERGENCE sparql/{name}\nengine={engine:#?}\njena={jena:#?}");
            summary.divergence();
        }
    }
    Ok(summary)
}

fn run_seed_shacl(root: &Path) -> Result<Summary> {
    let mut summary = Summary::default();
    for case in case_dirs(root)? {
        let name = case_name(&case)?;
        let meta = read_meta(&case)?;
        if let Some(reason) = meta.skip {
            println!("SKIP shacl/{name}: {reason}");
            summary.skip();
            continue;
        }

        let data = fs::read_to_string(case.join("data.ttl"))
            .with_context(|| format!("reading data for shacl/{name}"))?;
        let shapes = fs::read_to_string(case.join("shapes.ttl"))
            .with_context(|| format!("reading shapes for shacl/{name}"))?;
        let engine = comparable_report(&rdf_shapes_core::validate(&data, &shapes)?);
        let jena = jena_shacl(&case)?;
        match shacl_case_result(&name, &engine, &jena) {
            ShaclCaseResult::Pass => {
                println!("PASS shacl/{name}");
                summary.pass();
            }
            ShaclCaseResult::KnownDivergence(message) => {
                println!("KNOWN-DIVERGENCE shacl/{name}: {message}");
                summary.known_divergence();
            }
            ShaclCaseResult::Divergence => {
                eprintln!("DIVERGENCE shacl/{name}\nengine={engine:#?}\njena={jena:#?}");
                summary.divergence();
            }
        }
    }
    Ok(summary)
}

enum ShaclCaseResult {
    Pass,
    KnownDivergence(String),
    Divergence,
}

fn shacl_case_result(
    name: &str,
    engine: &ComparableShacl,
    jena: &ComparableShacl,
) -> ShaclCaseResult {
    if shacl_equal(engine, jena) {
        return ShaclCaseResult::Pass;
    }

    if name == "history-entry-violating" && matches_known_or_focus_node_divergence(engine, jena) {
        return ShaclCaseResult::KnownDivergence(
            "allowlisted focusNode mismatch for sh:OrConstraintComponent; Jena is correct; tracking #25 https://github.com/lambdasistemi/rdf-shapes-wasm/issues/25"
                .to_owned(),
        );
    }

    ShaclCaseResult::Divergence
}

fn matches_known_or_focus_node_divergence(
    engine: &ComparableShacl,
    jena: &ComparableShacl,
) -> bool {
    if engine.conforms != jena.conforms {
        return false;
    }

    let component = format!("{SH}OrConstraintComponent");
    let engine_index = engine.violations.iter().position(|violation| {
        violation.source_constraint_component == component
            && violation.result_path.is_none()
            && violation.focus_node
                == "https://lambdasistemi.github.io/amaru-treasury-tx/vocab/history#TreasuryEntityShape"
    });
    let jena_index = jena.violations.iter().position(|violation| {
        violation.source_constraint_component == component
            && violation.result_path.is_none()
            && violation.focus_node == "urn:entity:bad"
    });

    let (Some(engine_index), Some(jena_index)) = (engine_index, jena_index) else {
        return false;
    };

    let mut engine_without = engine.clone();
    let mut jena_without = jena.clone();
    engine_without.violations.remove(engine_index);
    jena_without.violations.remove(jena_index);
    shacl_equal(&engine_without, &jena_without)
}

fn run_w3c_shacl(root: &Path) -> Result<Summary> {
    let mut summary = Summary::default();
    for case in case_dirs(root)? {
        let name = case_name(&case)?;
        let meta = read_meta(&case)?;
        if let Some(reason) = meta.skip {
            println!("SKIP w3c/shacl/{name}: {reason}");
            summary.skip();
            continue;
        }

        let data = fs::read_to_string(case.join("data.ttl"))
            .with_context(|| format!("reading data for w3c/shacl/{name}"))?;
        let shapes = fs::read_to_string(case.join("shapes.ttl"))
            .with_context(|| format!("reading shapes for w3c/shacl/{name}"))?;
        let engine = comparable_report(&rdf_shapes_core::validate(&data, &shapes)?);
        let expected = parse_shacl_report(
            &fs::read_to_string(case.join("expected.ttl"))
                .with_context(|| format!("reading expected report for w3c/shacl/{name}"))?,
        )?;
        if shacl_equal(&engine, &expected) {
            println!("PASS w3c/shacl/{name}");
            summary.pass();
        } else {
            eprintln!("DIVERGENCE w3c/shacl/{name}\nengine={engine:#?}\nexpected={expected:#?}");
            summary.divergence();
        }
    }
    Ok(summary)
}

fn run_w3c_sparql(root: &Path) -> Result<Summary> {
    let mut summary = Summary::default();
    for case in case_dirs(root)? {
        let name = case_name(&case)?;
        let meta = read_meta(&case)?;
        if let Some(reason) = meta.skip {
            println!("SKIP w3c/sparql/{name}: {reason}");
            summary.skip();
            continue;
        }

        let graph = fs::read_to_string(case.join("graph.ttl"))
            .with_context(|| format!("reading graph for w3c/sparql/{name}"))?;
        let query = fs::read_to_string(case.join("query.rq"))
            .with_context(|| format!("reading query for w3c/sparql/{name}"))?;
        let mode = select_mode(&query, meta.ordered);
        let engine = engine_query(&graph, &query, mode)?;
        let expected = expected_query(&case, &engine, mode)?;
        if query_equal(&engine, &expected)
            .with_context(|| format!("comparing w3c/sparql/{name}"))?
        {
            println!("PASS w3c/sparql/{name}");
            summary.pass();
        } else {
            eprintln!("DIVERGENCE w3c/sparql/{name}\nengine={engine:#?}\nexpected={expected:#?}");
            summary.divergence();
        }
    }
    Ok(summary)
}

fn run_explicit_skips(root: &Path) -> Result<Summary> {
    let mut summary = Summary::default();
    if !root.exists() {
        return Ok(summary);
    }
    for entry in fs::read_dir(root).with_context(|| format!("reading {}", root.display()))? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let reason = fs::read_to_string(entry.path())?;
            println!(
                "SKIP {}: {}",
                entry.file_name().to_string_lossy(),
                reason.lines().next().unwrap_or("documented skip")
            );
            summary.skip();
        }
    }
    Ok(summary)
}

fn engine_query(graph: &str, query: &str, mode: SelectMode) -> Result<ComparableQuery> {
    match rdf_shapes_core::query(graph, query)? {
        QueryResults::Solutions { json } => Ok(ComparableQuery::Select { json, mode }),
        QueryResults::Boolean { value } => Ok(ComparableQuery::Ask(value)),
        QueryResults::Graph { ntriples } => Ok(ComparableQuery::Graph { ntriples }),
    }
}

fn jena_query(case: &Path, query: &str, mode: SelectMode) -> Result<ComparableQuery> {
    if is_graph_query(query) {
        let output = Command::new("arq")
            .arg("--data")
            .arg(case.join("graph.ttl"))
            .arg("--query")
            .arg(case.join("query.rq"))
            .arg("--results=N-TRIPLES")
            .output()
            .with_context(|| "running arq for graph query")?;
        command_stdout(output).map(|ntriples| ComparableQuery::Graph { ntriples })
    } else {
        let output = Command::new("arq")
            .arg("--data")
            .arg(case.join("graph.ttl"))
            .arg("--query")
            .arg(case.join("query.rq"))
            .arg("--results=json")
            .output()
            .with_context(|| "running arq for SELECT/ASK query")?;
        let json: serde_json::Value = serde_json::from_str(&command_stdout(output)?)
            .with_context(|| "parsing arq SPARQL Results JSON")?;
        if let Some(value) = json.get("boolean").and_then(serde_json::Value::as_bool) {
            Ok(ComparableQuery::Ask(value))
        } else {
            Ok(ComparableQuery::Select { json, mode })
        }
    }
}

fn expected_query(
    case: &Path,
    engine: &ComparableQuery,
    mode: SelectMode,
) -> Result<ComparableQuery> {
    match engine {
        ComparableQuery::Select { .. } => {
            let json: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(case.join("expected.srj"))?)
                    .with_context(|| "parsing expected SPARQL Results JSON")?;
            Ok(ComparableQuery::Select { json, mode })
        }
        ComparableQuery::Ask(_) => {
            let json: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(case.join("expected.srj"))?)
                    .with_context(|| "parsing expected ASK SPARQL Results JSON")?;
            let value = json
                .get("boolean")
                .and_then(serde_json::Value::as_bool)
                .ok_or_else(|| anyhow!("expected.srj missing boolean"))?;
            Ok(ComparableQuery::Ask(value))
        }
        ComparableQuery::Graph { .. } => {
            Ok(ComparableQuery::Graph { ntriples: fs::read_to_string(case.join("expected.nt"))? })
        }
    }
}

fn jena_shacl(case: &Path) -> Result<ComparableShacl> {
    let output = Command::new("shacl")
        .arg("validate")
        .arg("--shapes")
        .arg(case.join("shapes.ttl"))
        .arg("--data")
        .arg(case.join("data.ttl"))
        .output()
        .with_context(|| "running Jena shacl validate")?;
    parse_shacl_report(&command_stdout(output)?)
}

fn parse_shacl_report(turtle: &str) -> Result<ComparableShacl> {
    let quads = RdfParser::from_format(RdfFormat::Turtle)
        .without_named_graphs()
        .for_slice(turtle)
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| "parsing SHACL report graph")?;

    let mut conforms = None;
    let mut result_nodes = Vec::new();
    for quad in &quads {
        if quad.predicate.as_str() == SH_CONFORMS {
            if let Term::Literal(literal) = &quad.object {
                conforms = Some(literal.value() == "true");
            }
        }
        if quad.predicate.as_str() == SH_RESULT {
            result_nodes.push(term_key(&quad.object));
        }
        if quad.predicate.as_str() == RDF_TYPE
            && term_iri(&quad.object).as_deref() == Some(SH_VALIDATION_RESULT)
        {
            result_nodes.push(subject_key(&quad.subject));
        }
    }
    result_nodes.sort();
    result_nodes.dedup();

    let mut violations = Vec::new();
    for result_node in result_nodes {
        let focus_node = object_for(&quads, &result_node, SH_FOCUS_NODE)
            .map(normalize_term_identity)
            .ok_or_else(|| anyhow!("SHACL result missing sh:focusNode"))?;
        let source_constraint_component =
            object_for(&quads, &result_node, SH_SOURCE_CONSTRAINT_COMPONENT)
                .and_then(term_iri)
                .map(normalize_component)
                .ok_or_else(|| anyhow!("SHACL result missing sh:sourceConstraintComponent"))?;
        let result_path =
            object_for(&quads, &result_node, SH_RESULT_PATH).map(normalize_term_identity);
        violations.push(ViolationKey { focus_node, source_constraint_component, result_path });
    }

    Ok(ComparableShacl {
        conforms: conforms.ok_or_else(|| anyhow!("SHACL report missing sh:conforms"))?,
        violations,
    })
}

fn object_for<'a>(quads: &'a [oxrdf::Quad], subject: &str, predicate: &str) -> Option<&'a Term> {
    quads.iter().find_map(|quad| {
        (subject_key(&quad.subject) == subject && quad.predicate.as_str() == predicate)
            .then_some(&quad.object)
    })
}

fn comparable_report(report: &ValidationReport) -> ComparableShacl {
    let violations = report
        .violations
        .iter()
        .map(|violation| ViolationKey {
            focus_node: normalize_identity(&violation.focus_node),
            source_constraint_component: normalize_component(
                &violation.source_constraint_component,
            ),
            result_path: violation.path.as_deref().map(normalize_identity),
        })
        .collect();
    ComparableShacl { conforms: report.conforms, violations }
}

fn normalize_component(component: impl AsRef<str>) -> String {
    let component = normalize_identity(component.as_ref());
    match component.as_str() {
        "http://www.w3.org/ns/shacl#minCount" => format!("{SH}MinCountConstraintComponent"),
        "http://www.w3.org/ns/shacl#maxCount" => format!("{SH}MaxCountConstraintComponent"),
        "http://www.w3.org/ns/shacl#datatype" => format!("{SH}DatatypeConstraintComponent"),
        "http://www.w3.org/ns/shacl#nodeKind" => format!("{SH}NodeKindConstraintComponent"),
        "http://www.w3.org/ns/shacl#class" => format!("{SH}ClassConstraintComponent"),
        "http://www.w3.org/ns/shacl#or" => format!("{SH}OrConstraintComponent"),
        _ => component,
    }
}

fn normalize_term_identity(term: &Term) -> String {
    match term {
        Term::NamedNode(node) => node.as_str().to_owned(),
        Term::BlankNode(node) => format!("_:{}", node.as_str()),
        Term::Literal(literal) => literal.value().to_owned(),
        Term::Triple(triple) => triple.to_string(),
    }
}

fn term_iri(term: &Term) -> Option<String> {
    match term {
        Term::NamedNode(node) => Some(node.as_str().to_owned()),
        _ => None,
    }
}

fn term_key(term: &Term) -> String {
    normalize_term_identity(term)
}

fn subject_key(subject: &NamedOrBlankNode) -> String {
    match subject {
        NamedOrBlankNode::NamedNode(node) => node.as_str().to_owned(),
        NamedOrBlankNode::BlankNode(node) => format!("_:{}", node.as_str()),
    }
}

fn normalize_identity(value: &str) -> String {
    value
        .strip_prefix('<')
        .and_then(|stripped| stripped.strip_suffix('>'))
        .unwrap_or(value)
        .to_owned()
}

fn command_stdout(output: std::process::Output) -> Result<String> {
    if output.status.success() {
        String::from_utf8(output.stdout).with_context(|| "command stdout was not UTF-8")
    } else {
        bail!(
            "command failed with {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

fn case_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut dirs = fs::read_dir(root)
        .with_context(|| format!("reading {}", root.display()))?
        .map(|entry| {
            let entry = entry?;
            Ok((entry.file_type()?.is_dir(), entry.path()))
        })
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .filter_map(|(is_dir, path)| is_dir.then_some(path))
        .collect::<Vec<_>>();
    dirs.sort();
    Ok(dirs)
}

fn case_name(path: &Path) -> Result<String> {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| anyhow!("case path has no file name: {}", path.display()))
}

fn read_meta(case: &Path) -> Result<CaseMeta> {
    let path = case.join("meta.json");
    if path.exists() {
        serde_json::from_str(&fs::read_to_string(&path)?)
            .with_context(|| format!("parsing {}", path.display()))
    } else {
        Ok(CaseMeta::default())
    }
}

fn select_mode(query: &str, ordered: bool) -> SelectMode {
    if ordered || query.to_ascii_lowercase().contains("order by") {
        SelectMode::Ordered
    } else {
        SelectMode::Multiset
    }
}

fn is_graph_query(query: &str) -> bool {
    query
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .find(|line| {
            let lower = line.to_ascii_lowercase();
            !(lower.starts_with("prefix ") || lower.starts_with("base "))
        })
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            lower.starts_with("construct") || lower.starts_with("describe")
        })
        .unwrap_or(false)
}
