# rdf-shapes-wasm

A portable **SPARQL 1.1** query engine and **SHACL Core** validation
engine, written once in Rust and shipped to **multiple targets from one
source**: a browser WebAssembly bundle, a native C-ABI FFI library (the
server reuse path, called from Haskell via `foreign import ccall`), and
a self-contained native CLI.

This site documents the project: its motivations, the concepts it rests
on, how it is built, how to use it today, and the full Rust API
reference (rendered from the source itself).

It is part of the epic
[#1 — rdf-shapes-wasm](https://github.com/lambdasistemi/rdf-shapes-wasm/issues/1).

## Why this exists

Four motivations drive the project.

### 1. Kill the JVM / Apache Jena dependency

RDF tooling today leans on the JVM: querying and validating transaction
graphs typically means shelling out to Apache Jena's `arq` and `shacl`
command-line tools. That drags a whole Java runtime into every
consumer — slow to start, heavy to ship, awkward to embed. Replacing
Jena with one small, self-contained engine removes that dependency
entirely.

### 2. A client-side dashboard

A WebAssembly engine runs **in the browser** with no backend. A
dashboard can load an RDF graph, run SPARQL queries, and validate it
against SHACL shapes entirely client-side — no server round-trips, no
data leaving the page. The same engine that runs in CI runs in the
user's tab.

### 3. Reproducibility

Every artifact is built by Nix from a committed `Cargo.lock` with no
network access. The `.wasm` is byte-reproducible: two clean builds yield
an identical SHA-256, and `SHA256SUMS` ship with each release. A
consumer can rebuild the released bytes from source and verify they
match — audit-grade determinism, not a promise.

### 4. A self-contained release blob

The output is one `.wasm` file plus a thin JavaScript shim. No runtime
to install, no native libraries to link, no platform-specific binaries
to juggle. The release blob *is* the engine.

## The engine choice

Two mature Rust engines do the heavy lifting, both compiled into the
same wasm module:

- **[Oxigraph](https://github.com/oxigraph/oxigraph)** — SPARQL 1.1
  query.
- **[rudof](https://github.com/rudof-project/rudof)** — SHACL Core
  validation.

Rust compiles cleanly to `wasm32-unknown-unknown`, so the whole engine
fits in one portable artifact. Apache Jena remains in the picture only
as a *differential oracle* — the reference implementation we measure
parity against — never as a runtime dependency of the shipped artifact.

!!! note "Status"
    The evaluation surface is live: `query` (full SPARQL 1.1) and
    `validate` (SHACL Core) are exposed through the native CLI and the
    wasm bundle. See [Usage](usage.md). Full W3C-suite conformance and
    the Jena differential harness are tracked in the
    [issues](https://github.com/lambdasistemi/rdf-shapes-wasm/issues).

## Try it now

The [**Playground**](playground.md) runs SPARQL 1.1 and SHACL Core over
pasted Turtle entirely in your browser, via this same wasm engine — no
install, no server. It is the client-side dashboard above, live.

## Where to go next

- [Playground](playground.md) — run SPARQL + SHACL in the browser.
- [Concepts](concepts.md) — RDF, SPARQL 1.1, SHACL Core, and the wasm
  artifact, briefly.
- [Architecture](architecture.md) — the pure-core / thin-shells crate
  split, the Nix build, and the trust model.
- [Usage](usage.md) — building and running the CLI and the wasm bundle
  today.
- [API Reference](api-reference.md) — the full rustdoc, rendered inside
  this site.
