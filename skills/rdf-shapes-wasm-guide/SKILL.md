---
name: rdf-shapes-wasm-guide
description: >-
  Guide for working in the lambdasistemi/rdf-shapes-wasm repository — a
  portable SPARQL 1.1 query + SHACL Core validation engine written in
  Rust (rdf-shapes-core) and shipped to a browser wasm-bindgen bundle
  (rdf-shapes-wasm / wasm-pkg), a native C-ABI library (rdf-shapes-ffi /
  ffi-lib, librdf_shapes_ffi.so + rdf_shapes.h), and a native CLI
  (rdf-shapes-cli, the `rdf-shapes query` / `rdf-shapes validate`
  commands). Load this when the task mentions rdf-shapes, Oxigraph,
  rudof, SHACL, SPARQL, Turtle/.ttl, .rq queries, ValidationReport /
  Violation / QueryResults, ShapesError, the conformance / Jena
  differential harness (arq, shacl, rdf-shapes-conformance), the
  wasm-bindgen / wasm-opt / wasm-pkg build, the rdf_shapes_query /
  rdf_shapes_validate / rdf_shapes_string_free FFI symbols, the JSON
  `{"ok":…}` / `{"error":…}` envelope, the PureScript Halogen playground
  (app/, permalink #s=, ?ttl=&sparql=&shapes= machine path), or the Nix
  flake (just ci, nix flake check, nix build .#cli/.#wasm-pkg/.#ffi-lib).
---

# rdf-shapes-wasm guide

A portable SPARQL 1.1 + SHACL Core engine: one Rust core, three reuse
targets (browser wasm, native FFI lib, native CLI), plus a PureScript
browser playground. All logic lives in `rdf-shapes-core`; everything
else is a thin shell. Built and tested entirely with Nix.

## Repository map

| Path | Purpose |
|---|---|
| `crates/rdf-shapes-core/` | **All business logic.** Builds native and `wasm32`; no `wasm-bindgen`, no I/O. `query`/`validate` over in-memory Turtle. |
| `crates/rdf-shapes-wasm/` | `#[wasm_bindgen]` shims (`cdylib` + `rlib`) → the browser `.wasm`. |
| `crates/rdf-shapes-ffi/` | `extern "C"` shims (`cdylib`) → the native C-ABI lib. The only crate that allows `unsafe_code`. |
| `crates/rdf-shapes-cli/` | The native `rdf-shapes` binary — a thin `clap` front end. |
| `crates/rdf-shapes-conformance/` | Native-only conformance + Jena differential harness (test infra; not shipped). |
| `app/` | PureScript/Halogen browser playground over the wasm bundle. |
| `conformance/corpus/` | Committed conformance inputs (seed treasury + W3C SPARQL/SHACL cases). |
| `docs/` | MkDocs Material site sources. |
| `nix/` | Build modules: `crane.nix`, `packages.nix`, `checks.nix`, `apps.nix`, `conformance.nix`, `playground.nix`, `release.nix`, `api-docs.nix`, `toolchain.nix`. |
| `specs/`, `.specify/` | Spec-Kit feature specs and templates. |
| `justfile`, `flake.nix` | Thin wrappers; `just <recipe>` == `nix run .#<app>`. |

## Build, test, run

```bash
nix flake check          # the single gate (all checks below)
just ci                  # CI gate via nix run .#ci
just test                # unit tests (cargo-nextest)
just clippy              # clippy -D warnings
just fmt                 # format Rust in place (inside nix develop)
just conformance         # Jena differential + W3C conformance harness
nix build .#cli          # → ./result/bin/rdf-shapes
nix build .#wasm-pkg     # reproducible browser bundle
nix build .#ffi-lib      # librdf_shapes_ffi.{so,dylib} + include/rdf_shapes.h
nix build .#playground   # dist/{index.html,index.js}
nix build .#release-artifacts   # CLI tarball + npm .tgz + .wasm + FFI tarball + SHA256SUMS
```

The Rust toolchain is pinned in `rust-toolchain.toml` (channel 1.90.0,
`wasm32-unknown-unknown` target) and resolved through rust-overlay; crane
drives the Cargo build. The `wasm-bindgen` library version
(`Cargo.toml`) must equal the pinned `wasm-bindgen-cli` (`flake.nix`).

Two smoke harnesses are named operator follow-ups (not in the gate):
`crates/rdf-shapes-wasm/smoke/run-smoke.sh` (wasm↔CLI parity) and
`crates/rdf-shapes-ffi/smoke/run-smoke.sh` (Haskell `ccall` on GHC
9.12.3).

## Navigating the code

The engine is small — start in `crates/rdf-shapes-core/src/`:

- `lib.rs` — the façade: re-exports `query`, `validate`, `QueryResults`,
  `ValidationReport`, `Violation`, `ShapesError`, and `version()`.
- `query.rs` — `query(graph_ttl, sparql)`: loads Turtle into a fresh
  Oxigraph `Store`, evaluates SPARQL 1.1, returns typed `QueryResults`
  (SELECT → Query Results JSON; ASK → bool; CONSTRUCT/DESCRIBE →
  N-Triples).
- `validate.rs` — `validate(data_ttl, shapes_ttl)`: rudof Native (SHACL
  **Core**) engine; SHACL-SPARQL and remote graphs surface as
  `ShapesError::Unsupported`, never a silent pass. Violations are sorted
  for deterministic output.
- `results.rs` / `report.rs` — the serde types (`QueryResults`,
  `ValidationReport` + `Violation`).
- `error.rs` — `ShapesError` (`Parse` / `Query` / `Validation` /
  `Unsupported`).

Shells (each delegates to core, adds no logic):
`crates/rdf-shapes-cli/src/main.rs` (clap `query`/`validate`),
`crates/rdf-shapes-wasm/src/lib.rs` (`#[wasm_bindgen]` → `JsValue`),
`crates/rdf-shapes-ffi/src/lib.rs` (`extern "C"` → JSON envelope).
The differential harness:
`crates/rdf-shapes-conformance/src/main.rs` (drives the corpus, shells
out to `arq`/`shacl`) and `compare.rs` (semantic equality).
Playground: `app/src/Playground.purs` (Halogen UI),
`Permalink.purs` (`#s=` encoding), `MachinePath.purs`
(`?ttl=&sparql=&shapes=` params), `Examples.purs`, `FFI/RdfShapes.*`.

## Using the engine

**CLI** (after `nix build .#cli`):

```bash
./result/bin/rdf-shapes query    --graph graph.ttl --query q.rq
./result/bin/rdf-shapes validate --data data.ttl   --shapes shapes.ttl
```

`query` prints `{"kind":"solutions","json":{…}}` (SPARQL 1.1 Query
Results JSON, typed terms), or `{"kind":"boolean",…}` / `{"kind":"graph",
"ntriples":…}`. `validate` prints `{"conforms":bool,"violations":[…]}`.

**Browser wasm** (`nix build .#wasm-pkg`):

```js
import init, { start, query, validate } from "./rdf_shapes_wasm.js";
await init(); start();
const result = query(graphTtl, sparql);      // throws JS Error on failure
const report = validate(dataTtl, shapesTtl);
```

**Native FFI** (`nix build .#ffi-lib`) — C-ABI symbols
`rdf_shapes_query`, `rdf_shapes_validate`, `rdf_shapes_version`,
`rdf_shapes_string_free`. Each computing fn takes NUL-terminated UTF-8 C
strings and returns an owned JSON envelope: `{"ok":<result>}` on success,
`{"error":"<message>"}` on failure (NULL/non-UTF-8 inputs included). The
caller **must** free every returned pointer with `rdf_shapes_string_free`
exactly once.

```haskell
foreign import ccall "rdf_shapes_query"
  rdf_shapes_query :: CString -> CString -> IO CString
```

## Answering questions

- **"What is this / why?"** — `README.md` ("What is this") and
  `docs/index.md` ("Why this exists"). It is a portable SPARQL+SHACL
  engine replacing the JVM/Jena CLI dependency.
- **"How do I run a query / validate?"** — `docs/usage.md`; CLI flags in
  `crates/rdf-shapes-cli/src/main.rs`.
- **"What SPARQL/SHACL is supported?"** — full SPARQL 1.1 (Oxigraph);
  SHACL **Core** only (rudof Native) — SHACL-SPARQL and remote graphs are
  explicit `Unsupported` errors (`crates/rdf-shapes-core/src/validate.rs`,
  `docs/concepts.md`).
- **"How is it correct / why can it replace Jena?"** —
  `docs/conformance.md` + `conformance/README.md`: curated W3C suites
  against committed expected results, plus a Jena differential. One
  allowlisted known divergence (`sh:or` focus node, issue #25).
- **"How is the .wasm reproducible?"** — `docs/architecture.md`
  ("Reproducible wasm"); pinned `wasm-bindgen-cli` + `wasm-opt -Oz`,
  double-build SHA-256.
- **"How do I consume it downstream?"** — README "Downstream reuse" and
  `docs/usage.md`: flake outputs `wasm-pkg` and `ffi-lib`.
- **"How are releases cut?"** — README "Releasing": tag-driven
  (`just release X.Y.Z`, then push `vX.Y.Z`); `.github/workflows/release.yml`.
- **"What's the playground / permalink / machine path?"** —
  `docs/playground.md` and `app/README.md`.
