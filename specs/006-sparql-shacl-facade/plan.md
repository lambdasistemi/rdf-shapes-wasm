# Implementation Plan: SPARQL + SHACL evaluation façade

**Branch**: `feat/sparql-shacl-facade` | **Date**: 2026-06-09 | **Spec**: [spec.md](./spec.md)
**Issue**: #6 (epic #1)

## Summary

Promote spikes #3 (Oxigraph→wasm SPARQL) and #4 (rudof→wasm SHACL Core) into the
production `rdf-shapes-core` façade: a stable `query(graph, sparql)` and
`validate(data, shapes)` surface, full SPARQL 1.1 query + full SHACL Core,
exposed through the existing `rdf-shapes-wasm` (wasm-bindgen) crate and the
`rdf-shapes-cli` binary, shipping the combined reproducible `wasm-pkg`. The two
spikes already proved feasibility and the exact dependency configuration; this
feature hardens the surface (typed errors, structured results) and unifies the
two engines into one crate.

## Technical Context

**Language/Version**: Rust, pinned stable (currently 1.90 via `rust-toolchain.toml`)
**Primary Dependencies**:
- `oxigraph = "0.5.8"`, `default-features = false`, `features = ["js"]` (SPARQL 1.1, in-memory store)
- rudof SHACL family, version-pinned as a set (the 0.2.x tree is internally inconsistent across the `iri_s`→`rudof_iri` rename at 0.2.14): `shacl_validation = "=0.2.12"` (`default-features = false` → SHACL Core), `rudof_rdf = "=0.2.12"`, `sparql_service = "=0.2.12"`, `shacl_ir = "=0.2.9"`, `prefixmap = "=0.2.9"`, `mie = "=0.2.9"`
- `wasm-bindgen = "=0.2.108"` (bumped from scaffold's 0.2.100 — forced by oxigraph's `js-sys 0.3.85`; `wasm-bindgen-cli` pin in `flake.nix` bumped in lockstep to `wasm-bindgen-cli_0_2_108`)
- `serde` / `serde_json` / `serde-wasm-bindgen` (result + report marshalling), `thiserror` (core errors), `clap` (CLI)
**Storage**: none — fully in-memory; graphs/queries/shapes are Turtle strings
**Testing**: `cargo-nextest` unit tests (native) + a wasm smoke (Node) + a browser smoke page; conformance/differential vs Jena is #7
**Target Platform**: `wasm32-unknown-unknown` (browser, via wasm-bindgen) and the native host triple (CLI + the future server native lib)
**Project Type**: Rust workspace — portable core library + wasm shim + native CLI
**Performance Goals**: sub-second on a typical treasury graph (SC-005)
**Constraints**: byte-reproducible artifact; no network / no fs at evaluation; SHACL **Core** only; `getrandom` reconciled across both engines (oxigraph `js` feature vs rudof `wasm_js` + `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'` — standardize on one config that satisfies both)
**Scale/Scope**: treasury-sized RDF graphs (thousands of triples), arbitrary SPARQL 1.1 queries, SHACL Core shape sets

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **I. Pure Portable Core, Thin Shells** — `query`/`validate` land in
  `rdf-shapes-core` (builds native + wasm32, no I/O/threads/fs/sockets/wasm-bindgen);
  `rdf-shapes-wasm` and `rdf-shapes-cli` stay thin marshalling shells. Both
  engines are configured `default-features=false` to keep native-only paths
  (tokio/reqwest/rocksdb/fs) out of the wasm build.
- [x] **II. Reproducible, Content-Addressed Artifacts** — same crane→pinned
  `wasm-bindgen-cli`→`wasm-opt` pipeline; `wasm-bindgen` lib == cli (both
  0.2.108) bumped in one commit; `Cargo.lock` committed; double-build SHA
  verified. Engine version family pinned exactly (`=`).
- [x] **III. Conformance & Differential Correctness** — this feature ships
  evaluation + a smoke level of self-checking (the spike smokes promoted to unit
  tests); full W3C-suite + Jena-differential parity is #7, explicitly. No silent
  unsupported gaps — SHACL-SPARQL and remote graphs are reported as unsupported.
- [x] **IV. Nix-First Single Gate** — clippy `-D warnings`, fmt, nextest,
  cargo-deny, doc all stay green under `nix flake check` == `just ci` == CI;
  `deny.toml` license allow-list widened deliberately for the oxigraph/rudof
  trees.
- [x] **V. Disciplined Delivery** — one issue-backed PR (#6), linear history,
  Conventional Commits, small slices, `Cargo.lock` + cargo-deny pinning.

No violations → Complexity Tracking empty.

## Project Structure

### Documentation (this feature)

```text
specs/006-sparql-shacl-facade/
├── spec.md
├── plan.md            # this file
├── tasks.md           # /speckit.tasks output
└── checklists/requirements.md
```

### Source Code (repository root)

```text
crates/
├── rdf-shapes-core/        # the façade — all logic, native + wasm32
│   └── src/
│       ├── lib.rs          # re-exports query/validate + error types
│       ├── error.rs        # `thiserror` enums: ParseError, QueryError, ValidationError, Unsupported
│       ├── query.rs        # oxigraph in-memory Store; load Turtle; run SPARQL 1.1; serialize results
│       └── validate.rs     # rudof shacl_validation (Core); load data+shapes; structured report
├── rdf-shapes-wasm/        # thin #[wasm_bindgen] shims over core (serde-wasm-bindgen)
│   └── src/lib.rs          # query(graph, sparql) / validate(data, shapes) -> JsValue
└── rdf-shapes-cli/         # `rdf-shapes query` / `rdf-shapes validate` subcommands over core
    └── src/main.rs
nix/
├── crane.nix               # add getrandom RUSTFLAGS reconciliation if needed for the wasm artifacts
└── packages.nix            # wasm-pkg now carries the real engines; wasm-bindgen-cli pin -> 0_2_108
```

**Structure Decision**: Keep the scaffold's three-crate shape (Constitution I).
All new logic is in `rdf-shapes-core`; the wasm and CLI crates only marshal.

### Result & report shapes (Phase 1 design)

- **SPARQL results**: serialize via Oxigraph's native SPARQL Query Results JSON
  writer (structured term/datatype), not ad-hoc `to_string()` — so consumers get
  proper typing (improves on the spike's lexical-form hack).
- **SHACL report**: a serde struct `{ conforms: bool, violations: [{ focus_node,
  path?, value?, source_constraint_component, message, severity }] }`, mapped
  from rudof's validation result.
- **Errors**: a single `thiserror` enum with variants for parse/query/validation
  failures and `Unsupported(feature)` (SHACL-SPARQL, remote graphs), surfaced to
  wasm as a rejected/Err `JsValue` and to the CLI as a non-zero exit + stderr.

## Complexity Tracking

No constitution violations — table intentionally empty.

## Phase notes

- **Phase 0 (research)**: already done — spikes #3/#4/#5 (verdicts on issues
  #3/#4/#5; reference branches `spike/sparql-wasm`, `spike/shacl-wasm`,
  `spike/server-host`). The exact dep config above comes from those verdicts.
- **Phase 1 (design)**: the API surface, result/report structs, and error model
  above. Re-checked against the Constitution — still all green.
- **Phase 2 (tasks)**: see `tasks.md`.
