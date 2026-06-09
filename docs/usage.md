# Usage

How to build and run the project. Everything runs through Nix, so the
commands are identical locally and in CI.

The engine exposes two capabilities over in-memory Turtle:

- **`query`** — full SPARQL 1.1 query via Oxigraph's in-memory store.
- **`validate`** — SHACL Core validation via rudof.

Both are reachable through the native CLI and the browser (wasm)
surface, and return equivalent results for equivalent inputs.

## Building

Build the deliverables straight from the flake:

```bash
# Native CLI (the `rdf-shapes` executable)
nix build .#cli

# Portable core library (native build)
nix build .#lib

# Reproducible, npm-shaped wasm bundle
nix build .#wasm-pkg

# CLI tarball + npm .tgz + bare .wasm + SHA256SUMS
nix build .#release-artifacts

# This documentation's Rust API reference (rustdoc HTML)
nix build .#api-docs
```

## The CLI

After `nix build .#cli`, the binary is at `result/bin/rdf-shapes`:

```console
$ ./result/bin/rdf-shapes --help
SPARQL 1.1 query and SHACL Core validation over RDF graphs

Usage: rdf-shapes <COMMAND>

Commands:
  query     Run a SPARQL 1.1 query over a Turtle graph, printing JSON
  validate  Validate a Turtle data graph against SHACL Core shapes
  help      Print this message or the help of the given subcommand(s)
```

### `query`

Given `graph.ttl`:

```turtle
@prefix cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#> .
<urn:tx:aaaa> a cardano:Transaction .
<urn:tx:bbbb> a cardano:Transaction .
<urn:tx:cccc> a cardano:Transaction .
```

and `tx-count.rq`:

```sparql
PREFIX cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#>
SELECT (COUNT(DISTINCT ?tx) AS ?transactions)
WHERE { ?tx a cardano:Transaction . }
```

```console
$ ./result/bin/rdf-shapes query --graph graph.ttl --query tx-count.rq
{
  "kind": "solutions",
  "json": {
    "head": { "vars": [ "transactions" ] },
    "results": {
      "bindings": [
        {
          "transactions": {
            "type": "literal",
            "value": "3",
            "datatype": "http://www.w3.org/2001/XMLSchema#integer"
          }
        }
      ]
    }
  }
}
```

SELECT results are emitted as the SPARQL 1.1 Query Results JSON
document, so each binding keeps its term kind and datatype. ASK returns
`{ "kind": "boolean", "value": true }`; CONSTRUCT/DESCRIBE returns
`{ "kind": "graph", "ntriples": "…" }`.

### `validate`

```console
$ ./result/bin/rdf-shapes validate --data data.ttl --shapes history-entry.shacl.ttl
{
  "conforms": false,
  "violations": [
    {
      "focus_node": "…#entry-bad",
      "path": "…#slot",
      "value": "\"not-a-number\"",
      "source_constraint_component": "http://www.w3.org/ns/shacl#datatype",
      "message": "Expected datatype: xsd:integer",
      "severity": "Violation"
    }
  ]
}
```

A conforming graph yields `{ "conforms": true, "violations": [] }`.
Violations are sorted on a stable key, so the report is deterministic.
Shapes that require SHACL-SPARQL or remote graphs are reported as an
`unsupported` error rather than silently passing.

## The wasm bundle

`nix build .#wasm-pkg` produces a web-target bundle (`.wasm`, the
`wasm-bindgen` JS shim, TypeScript types, and a `package.json`). The
exported functions mirror the CLI and return the same JSON structures
as JS objects:

```js
import init, { start, query, validate } from "./rdf_shapes_wasm.js";

await init();
start(); // installs the panic hook

const result = query(graphTtl, sparql);
// { kind: "solutions", json: { head: …, results: … } }

const report = validate(dataTtl, shapesTtl);
// { conforms: false, violations: [ … ] }
```

On a parse, query, validation, or unsupported-feature error, the
functions throw a JS `Error` carrying the structured message.

A headless parity smoke under `crates/rdf-shapes-wasm/smoke/` runs the
same query and validation through both the wasm bundle and the CLI and
asserts the outputs match:

```bash
nix develop -c crates/rdf-shapes-wasm/smoke/run-smoke.sh
```

## Reproducibility

The `.wasm` is byte-reproducible. Build it twice and compare:

```bash
nix build .#wasm-pkg -o wp-a
nix build .#wasm-pkg --rebuild -o wp-b
sha256sum wp-a/rdf_shapes_wasm_bg.wasm wp-b/rdf_shapes_wasm_bg.wasm
# identical SHA-256
```

## The gate

| Command | What it does |
|---|---|
| `nix flake check` | The single gate: clippy (`-D warnings`), rustfmt, nextest, cargo-deny, rustdoc (`-D warnings`). |
| `just ci` | The same gate via `nix run .#ci`. |
| `just build` | Native CLI + core library. |
| `just test` | Unit tests (cargo-nextest). |
| `just clippy` | Clippy with warnings denied. |
| `just fmt-check` | rustfmt in check mode. |
| `just wasm` | The reproducible wasm bundle. |

Run `just --list` for the full set. A change is not "done" until
`nix flake check` is green.

## What is out of scope here

Full W3C SPARQL 1.1 / SHACL conformance suites and byte-for-byte
differential parity against Apache Jena are delivered by the separate
testing feature
([#7](https://github.com/lambdasistemi/rdf-shapes-wasm/issues/7)); this
feature delivers the evaluation surface plus the smoke level of
self-checking above.
