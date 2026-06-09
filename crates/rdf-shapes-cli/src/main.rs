//! Native CLI shell over [`rdf_shapes_core`].
//!
//! A thin `clap` front end that marshals command-line arguments into
//! calls on the portable core and prints the result. It carries no
//! business logic of its own; the heavy conformance and differential
//! runners arrive with later work.

use clap::{Parser, Subcommand};

/// `rdf-shapes` command-line interface.
#[derive(Debug, Parser)]
#[command(name = "rdf-shapes", version, about = "Trivial scaffold CLI over rdf-shapes-core")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The subcommands exposed by the scaffold CLI.
#[derive(Debug, Subcommand)]
enum Command {
    /// Print the crate version reported by the core.
    Version,
    /// Echo a value back through the core's `ping`.
    Ping {
        /// The text to echo.
        input: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Version => println!("{}", rdf_shapes_core::version()),
        Command::Ping { input } => {
            println!("{}", rdf_shapes_core::ping(&input));
        }
    }
    Ok(())
}
