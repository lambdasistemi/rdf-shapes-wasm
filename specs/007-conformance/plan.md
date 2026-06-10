# Implementation Plan: Conformance + Jena differential harness

**Branch**: `feat/conformance` | **Date**: 2026-06-10 | **Spec**: [spec.md](./spec.md)
**Issue**: #7 (epic #1)

## Summary

A corpus-driven harness that (a) runs the engine and Apache Jena (`arq`/`shacl`)
over a treasury seed corpus and asserts semantic equivalence (the Jena-parity
trust anchor), and (b) runs the engine against a curated set of W3C SPARQL 1.1 +
SHACL Core cases vs their official expected results. Both gate under
`nix flake check`. This makes Constitution Principle III concrete.

## Technical Context

**Language**: Rust (the harness) + Nix (the gate); Apache Jena (JVM) as oracle.
**Primary Dependencies**: `rdf-shapes-core` (engine under test), `serde_json`
(SPARQL Results JSON), `oxrdf`/`oxttl` or core helpers (parse Turtle/N-Triples
for triple-set + SHACL-report comparison); `apache-jena` (nixpkgs 5.x, provides
`arq` + `shacl`), pinned via the flake.
**Testing**: a `rdf-shapes-conformance` runner exercised by a `conformance` Nix
check; plus a negative test proving the gate fails on a wrong answer.
**Target Platform**: native (the harness + Jena run on the host/CI, not wasm).
**Project Type**: Rust workspace — add a conformance harness crate + corpus.
**Performance Goals**: the seed-corpus + curated-W3C run completes well within a
normal CI step (seconds-to-low-minutes incl. JVM startup).
**Constraints**: reproducible (corpus + W3C cases committed; Jena pinned);
semantic comparison (normalize ordering, blank nodes, typed literals); no silent
skips.
**Scale/Scope**: ~8 treasury items + a curated W3C subset now; corpus-extensible.

## Constitution Check

- [x] **I. Pure Portable Core, Thin Shells** — the harness is test infrastructure
  over `rdf-shapes-core`; it adds **no** engine logic. (It may live in a native
  `rdf-shapes-conformance` crate that needn't compile to wasm — it's a dev/test
  tool, not part of the shipped engine; documented in Complexity Tracking.)
- [x] **II. Reproducible, Content-Addressed Artifacts** — the corpus and the W3C
  cases are committed; `apache-jena` is pinned by the flake; the check runs in the
  Nix sandbox.
- [x] **III. Conformance & Differential Correctness (NON-NEGOTIABLE)** — this
  feature *is* Principle III: W3C suites + the Jena differential on the seed
  corpus. Out-of-scope features are explicit skips.
- [x] **IV. Nix-First Single Gate** — `checks.conformance` joins `nix flake check`
  == `just ci` == CI; a `just conformance` / `nix run .#conformance` mirror.
- [x] **V. Disciplined Delivery** — one issue-backed PR (#7), Conventional
  Commits, committed corpus, pinned oracle.

## Project Structure

```text
conformance/
├── corpus/
│   ├── sparql/<case>/{graph.ttl, query.rq}          # differential vs Jena
│   ├── shacl/<case>/{data.ttl, shapes.ttl}          # differential vs Jena
│   └── w3c/
│       ├── sparql/<case>/{..., expected.srj}        # vs official expected
│       └── shacl/<case>/{data.ttl, shapes.ttl, expected.ttl}
└── README.md                                         # how to add a case + regen
crates/rdf-shapes-conformance/                         # native harness (bin)
└── src/main.rs        # iterate corpus → run engine + (Jena | expected) → compare → summary/exit
nix/
├── conformance.nix    # checks.conformance: runCommand w/ apache-jena + the harness over ${src}/conformance
flake.nix              # + packages.conformance (the runner) + checks.conformance
justfile               # + `conformance` recipe
```

**Structure Decision**: a dedicated **native** harness crate keeps Jena/JVM and
comparison machinery out of the engine crates. The treasury queries/shapes are
copied into `conformance/corpus/` from `amaru-treasury-tx` (vendored, with a
note on provenance) so the repo is self-contained.

### Comparison semantics (the core of the harness)

- **SELECT**: parse both sides as SPARQL Results JSON → compare as a multiset of
  rows; each binding compared on (term-kind, value, datatype, lang); blank nodes
  compared structurally (canonical relabeling), not by label. Respect order only
  when the query has `ORDER BY`.
- **ASK**: boolean equality. **CONSTRUCT/DESCRIBE**: canonical triple-set equality
  (parse N-Triples; blank-node canonicalization).
- **SHACL**: compare `conforms`; compare the violation set keyed by
  (focus node, source constraint component, result path), ignoring message text.
  Engine side = `ValidationReport`; Jena side = parse the `shacl` report graph.
- **W3C "expected" mode**: same comparators, but the oracle is the committed
  expected result instead of live Jena.

### Jena invocation

`arq --data graph.ttl --query q.rq --results=json` for SPARQL; `shacl validate
--shapes shapes.ttl --data data.ttl` for SHACL (parse the emitted report graph).
Jena is found on `PATH`, provided by the `conformance` check's `nativeBuildInputs`
(`pkgs.apache-jena`).

### The gate

`checks.conformance` is a `runCommand` (per the `/nix` skill: real sandboxed
check, not a writeShellApplication shell) that puts the crane-built harness +
`apache-jena` on PATH and runs it over `${src}/conformance`, failing on any
divergence and printing the pass/skip/divergence summary. Network-isolated, so
all W3C cases are committed (no fetch at check time).

## Complexity Tracking

| Violation | Why needed | Simpler alternative rejected because |
|---|---|---|
| A 5th crate (`rdf-shapes-conformance`) that isn't wasm-portable | The harness shells to Jena + does heavy comparison; it must not pollute the portable engine crates | Putting it in `rdf-shapes-core` would drag test/JVM concerns into the wasm-portable core (violates Principle I); a separate native dev crate is the clean boundary |

## Phase notes

- W3C cases: curate a representative handful from the W3C SPARQL 1.1 query test
  suite and the W3C SHACL test suite (commit the case files + expected results;
  record provenance/commit in `conformance/README.md`). Full manifest ingestion
  is a future expansion.
- Divergences found against Jena are **documented and triaged in the PR** before
  this is considered the cutover license — a real divergence is a finding, not an
  auto-fail to paper over.
