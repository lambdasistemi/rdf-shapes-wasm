<!--
SYNC IMPACT REPORT
==================
Version change: (none) → 1.0.0  (initial ratification)
Bump rationale: First constitution for the repository; MAJOR baseline.

Principles defined:
  I.   Pure Portable Core, Thin Shells
  II.  Reproducible, Content-Addressed Artifacts
  III. Conformance & Differential Correctness (NON-NEGOTIABLE)
  IV.  Nix-First Single Gate
  V.   Disciplined Delivery

Sections defined:
  - Technology Constraints
  - Development Workflow & Quality Gates
  - Governance

Templates reviewed:
  ✅ .specify/templates/plan-template.md — Constitution Check gate filled
  ✅ .specify/templates/spec-template.md — no constitution-driven change needed
  ✅ .specify/templates/tasks-template.md — no constitution-driven change needed

Deferred TODOs: none.
-->

# rdf-shapes-wasm Constitution

`rdf-shapes-wasm` is a Rust workspace that compiles a SPARQL 1.1 query engine
(Oxigraph) and a SHACL Core validation engine (rudof) to one WebAssembly
artifact, runnable in the browser, on the server, in CI, and as a self-contained
release blob. These principles are non-negotiable defaults; deviations require an
explicit, justified entry in the plan's Complexity Tracking.

## Core Principles

### I. Pure Portable Core, Thin Shells

The crate `rdf-shapes-core` holds ALL business logic and MUST compile for both
the native host triple and `wasm32-unknown-unknown`. It MUST NOT depend on
`wasm-bindgen`, perform direct I/O, spawn threads, touch the filesystem, or open
sockets — anything that does not build for `wasm32` does not belong in core. The
crates `rdf-shapes-wasm` (a `cdylib` carrying only `#[wasm_bindgen]` shims) and
`rdf-shapes-cli` (a native binary) are THIN shells: they marshal inputs/outputs
to and from core and add nothing else.

Rationale: one engine must run unchanged across browser, server-wasm-host, and
native CLI. Keeping host concerns out of core is what makes "build once, run
everywhere" true rather than aspirational.

### II. Reproducible, Content-Addressed Artifacts

Every released artifact — above all the `.wasm` — MUST be produced by Nix from a
committed `Cargo.lock`, with no network access at build time. `wasm-pack` is
forbidden (it fetches its own toolchain). The `.wasm` is built by crane and
finalized by a pinned `wasm-bindgen-cli` + `wasm-opt`; the `wasm-bindgen` library
version MUST equal the pinned `wasm-bindgen-cli` version, changed together in one
commit. Reproducibility MUST be verifiable: two clean builds of `wasm-pkg` MUST
yield an identical SHA-256, and `SHA256SUMS` ship with each release.

Rationale: audit-grade determinism is a stated project goal — a consumer must be
able to rebuild the released bytes from source and get the same artifact.

### III. Conformance & Differential Correctness (NON-NEGOTIABLE)

Correctness of the engine is established by evidence, not assertion:
(a) the W3C SPARQL 1.1 query test suite and the W3C SHACL test suite, and
(b) a differential harness that runs the same graph + query/shape through both
Apache Jena (`arq`/`shacl`, the reference oracle) and this engine and asserts
identical results over a seed corpus. The engine MUST achieve parity on the seed
corpus before it may replace Jena in any consumer. Tests precede behavior (TDD)
wherever practical; any known-unsupported feature is an explicit, documented skip,
never silent. There is no formal-proof (Lean) gate in this project — packaging an
existing engine has no novel state-machine invariant to prove; conformance and
differential parity ARE the arbiter of correctness.

Rationale: we are replacing a trusted engine. Measured parity against it is the
only thing that licenses deleting it.

### IV. Nix-First Single Gate

All building, testing, linting, and formatting run through Nix. `nix flake check`
MUST be the single source of truth and MUST equal `just ci` and the CI gate —
the same command passes locally and in CI, with no environment drift. Clippy runs
with `-D warnings` (a warning fails the build), `rustfmt` is enforced in check
mode, `cargo-deny` is enforced, and `cargo doc` runs with `-D warnings`. A change
is not "done" until `nix flake check` is green; "it compiles" is not sufficient.

Rationale: identical environments everywhere eliminate the "works on my machine"
class of failure and make CESI red/green meaningful.

### V. Disciplined Delivery

Work is delivered through issue-backed pull requests; direct pushes to `main` are
forbidden (the only exception is the documented repo-bootstrap stub). History is
linear — rebase-merge only, never squash, never merge commits. Commits follow
Conventional Commits, are small and single-concern, and are bisect-safe. The
supply chain is pinned: `Cargo.lock` is committed, `cargo-deny` gates licenses
and sources, and every dependency source is explicit. PR descriptions are living
documents kept current as a tour of the change.

Rationale: a bisectable, reviewable, reproducible history is worth more than
momentary convenience, especially for an audit-facing tool.

## Technology Constraints

- **Language**: Rust, pinned to an explicit STABLE channel in
  `rust-toolchain.toml` (read by `rust-overlay`); the workspace `rust-version`
  (MSRV) MUST be ≤ that pin. Bumps are deliberate.
- **Nix build**: `crane` + `rust-overlay`; `flake.nix` stays thin and imports
  `nix/*.nix`. No `buildRustPackage`/`naersk`.
- **Targets**: native host triple and `wasm32-unknown-unknown`.
- **Engines**: Oxigraph (SPARQL 1.1) and rudof / `shacl_validation` (SHACL Core)
  are the only graph engines. Apache Jena is the differential oracle ONLY — it is
  never a runtime dependency of the shipped artifact.
- **De-risking**: a heavy or wasm-uncertain dependency (Oxigraph, rudof) is first
  proven through a throwaway spike before it is wired into the production façade.
- **Lints**: `[workspace.lints]` set `clippy::all = deny` and forbid
  `unsafe_code`.

## Development Workflow & Quality Gates

- Every ticket goes through Spec-Driven Development (`speckit`): specify → plan →
  tasks → implement. This constitution is the gate the plan's Constitution Check
  evaluates against.
- Spikes are throwaway and exempt from the conformance gate (Principle III) and
  from the production lint surface; they exist only to retire risk and produce a
  written verdict.
- CI MUST be green before merge; every PR is labeled and assigned.
- Significant deletions or scope changes stop for review before proceeding.

## Governance

This constitution supersedes other practices on conflict. Amendments are made by
pull request that edits this file and bumps the version per semantic versioning:
MAJOR for incompatible principle removals/redefinitions, MINOR for a new principle
or materially expanded section, PATCH for clarifications. Every plan MUST complete
its Constitution Check against the current version before Phase 0, and re-check
after design; unavoidable violations MUST be recorded and justified in the plan's
Complexity Tracking. Compliance is verified at PR review.

**Version**: 1.0.0 | **Ratified**: 2026-06-09 | **Last Amended**: 2026-06-09
