# Feature Specification: Conformance + Jena differential harness

**Feature Branch**: `feat/conformance`
**Created**: 2026-06-10
**Status**: Draft
**Issue**: #7 (epic #1)
**Input**: The trust anchor that licenses replacing Apache Jena — W3C SPARQL 1.1
+ SHACL Core conformance, and a differential harness asserting the engine agrees
with Jena (`arq`/`shacl`) on a treasury-tx seed corpus.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Differential parity vs Jena on the seed corpus (Priority: P1)

A maintainer needs evidence that this engine returns the *same answers* as Apache
Jena for the queries and shapes the treasury actually uses, so Jena can be
deleted from the server with confidence. A harness runs each corpus item through
**both** the engine and Jena and asserts the results are semantically equivalent.

**Why this priority**: This is the whole point of #7 — it is what licenses the
Jena cutover in `amaru-treasury-tx`. Without it, replacing Jena is a leap of
faith.

**Independent Test**: Run the harness over the seed corpus (the treasury named
queries + the `history-entry`/`indexed-tx-body` shapes over sample graphs); it
passes only if the engine and Jena agree on every item.

**Acceptance Scenarios**:

1. **Given** a corpus SELECT query + graph, **When** run through the engine and
   `arq`, **Then** the two result sets are equal as binding multisets (and as
   ordered lists when the query has `ORDER BY`).
2. **Given** a corpus ASK/CONSTRUCT, **When** run through both, **Then** the
   boolean / the result triple-set match.
3. **Given** a corpus data graph + SHACL shapes, **When** validated by the engine
   and by `shacl`, **Then** the `conforms` verdict matches and the violation sets
   match keyed by (focus node, constraint component, path).
4. **Given** any divergence, **When** the harness runs, **Then** it fails loudly
   and reports the diverging item + both outputs.

---

### User Story 2 - W3C standards conformance (Priority: P2)

A maintainer wants assurance the engine conforms to the published standards, not
just to Jena. The engine is run against a curated set of **W3C SPARQL 1.1 query**
and **W3C SHACL Core** test cases with their official expected results.

**Why this priority**: Standards conformance backs the correctness claim beyond
"matches Jena"; bounded to a representative, extensible set because the upstream
engines (Oxigraph/rudof) already carry full-suite conformance and the
differential (US1) covers our real usage.

**Independent Test**: Run the engine against the committed W3C cases; each
passes against its expected result, or is an explicit, documented skip.

**Acceptance Scenarios**:

1. **Given** a W3C SPARQL 1.1 query test (manifest: query + data + expected
   results), **When** evaluated, **Then** the engine's result equals the expected
   result (semantically).
2. **Given** a W3C SHACL Core test (data + shapes + expected report), **When**
   validated, **Then** the conformance + violations match the expected report.
3. **Given** a feature outside scope (SHACL-SPARQL, federation, remote graphs),
   **When** encountered, **Then** it is an explicit skip with a recorded reason —
   never a silent pass or failure.

---

### User Story 3 - Wired into the gate (Priority: P2)

Both harnesses run under the project's single Nix gate / CI, so a regression that
breaks Jena-parity or standards conformance fails the build.

**Acceptance Scenarios**:

1. **Given** the harness, **When** `nix flake check` runs, **Then** the
   conformance + differential checks execute and gate the result.

---

### Edge Cases

- Result ordering: compare as multisets unless the query is ordered; normalize
  blank-node labels (compare structurally, not by label).
- Typed literals: compare on (lexical value, datatype) so `"3"^^xsd:integer`
  matches across engines.
- Jena and the engine phrase reports/messages differently — compare **semantics**
  (verdict + violation identity), not raw report text.
- A corpus item Jena rejects but is out of the engine's scope (or vice versa) is
  recorded as a documented divergence/skip, not a silent pass.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A **seed corpus** MUST exist: the treasury named queries
  (`tx-count`, `asset-flow`, `spend-edges`, `entity-occurrences`,
  `history-entries`, `address-resolution`) + the `history-entry` and
  `indexed-tx-body` shapes, with sample graphs (conforming and violating), plus a
  few additional SPARQL-1.1 / SHACL-Core cases.
- **FR-002**: The differential harness MUST run each corpus item through both the
  engine and Apache Jena (`arq` for SPARQL, `shacl` for SHACL) and assert
  **semantic equivalence** (binding multiset / boolean / triple-set; SHACL
  verdict + violation set).
- **FR-003**: On any divergence the harness MUST fail and report the item and
  both outputs.
- **FR-004**: A curated set of **W3C SPARQL 1.1 + SHACL Core** conformance cases
  MUST be run against their official expected results.
- **FR-005**: Out-of-scope features MUST be explicit, documented skips (no silent
  pass/fail). A summary of pass/skip counts MUST be produced.
- **FR-006**: Both harnesses MUST be wired into `nix flake check` / CI so
  regressions gate.

### Key Entities

- **Corpus item**: a named case — kind (sparql/shacl), inputs (graph; query or
  shapes), and (for W3C) an expected result.
- **Comparison**: the normalized, semantic equality used per result kind.
- **Report**: per-item pass/skip/divergence + an overall summary.

## Success Criteria *(mandatory)*

- **SC-001**: 100% of the treasury seed-corpus items agree between the engine and
  Jena (or any divergence is documented and triaged before the Jena cutover).
- **SC-002**: The curated W3C SPARQL 1.1 + SHACL Core cases pass (or are
  documented skips); the pass/skip summary is produced.
- **SC-003**: The harnesses run under `nix flake check`; a deliberately injected
  wrong-answer makes the check fail (the gate is real, not a no-op).
- **SC-004**: The harness is corpus-driven, so adding a case is dropping files in,
  not editing harness code.

## Assumptions

- "Parity" means **semantic equivalence on normalized output**, not byte-identical
  reports (engines format differently). This refines the epic's "byte-for-byte"
  wording.
- **Apache Jena 5.x** (`apache-jena` in nixpkgs; provides `arq`/`shacl`) is the
  oracle, pinned via the flake.
- W3C conformance is a **curated, extensible subset** in this feature; full
  manifest-driven ingestion of the entire W3C suites is a future expansion (the
  upstream engines already carry full-suite conformance).
- SHACL is **Core only** (the engine's scope); SHACL-SPARQL is an explicit skip.
- The engine under test is `rdf-shapes-core` (exercised via the `rdf-shapes` CLI
  and/or Rust harness); this feature adds tests/harness, not engine behavior.
