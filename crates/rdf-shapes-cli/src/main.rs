//! Native CLI shell over [`rdf_shapes_core`].
//!
//! A thin `clap` front end that reads Turtle / query / shapes from
//! files, calls the portable core, and prints the result as JSON. It
//! carries no business logic of its own; a [`rdf_shapes_core::Shapes
//! Error`] is surfaced as a non-zero exit plus a message on stderr.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

/// `rdf-shapes` command-line interface.
#[derive(Debug, Parser)]
#[command(
    name = "rdf-shapes",
    version,
    about = "SPARQL 1.1 query and SHACL Core validation over RDF graphs"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The subcommands exposed by the CLI.
#[derive(Debug, Subcommand)]
enum Command {
    /// Run a SPARQL 1.1 query over a Turtle graph, printing JSON.
    Query {
        /// Path to the RDF graph (Turtle).
        #[arg(long)]
        graph: PathBuf,
        /// Path to the SPARQL 1.1 query.
        #[arg(long)]
        query: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Query { graph, query } => run_query(&graph, &query),
    }
}

/// Loads the graph and query files, evaluates, and prints the typed
/// results as pretty JSON.
fn run_query(graph: &PathBuf, query: &PathBuf) -> Result<()> {
    let graph_ttl = fs::read_to_string(graph)
        .with_context(|| format!("reading graph {}", graph.display()))?;
    let sparql = fs::read_to_string(query)
        .with_context(|| format!("reading query {}", query.display()))?;

    let results = rdf_shapes_core::query(&graph_ttl, &sparql)
        .context("SPARQL query failed")?;
    let json = serde_json::to_string_pretty(&results)
        .context("serializing results")?;
    println!("{json}");
    Ok(())
}
