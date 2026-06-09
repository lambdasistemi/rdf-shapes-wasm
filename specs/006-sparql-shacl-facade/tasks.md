# Tasks: SPARQL + SHACL evaluation façade

**Input**: [spec.md](./spec.md), [plan.md](./plan.md) · **Issue**: #6
**Tests**: INCLUDED — the constitution (III, conformance) and the spec's
acceptance scenarios require them. Write the test, watch it fail, then implement.

## Format: `[ID] [P?] [Story] Description`
- **[P]** = parallelizable (different files, no dependency)

---

## Phase 1: Setup (shared dependency configuration)

- [ ] T001 Add engine deps to root `Cargo.toml` `[workspace.dependencies]`: `oxigraph` (`=0.5.8`, `default-features=false`, `features=["js"]`), the rudof SHACL family pinned exactly (`shacl_validation=0.2.12` `default-features=false`, `rudof_rdf=0.2.12`, `sparql_service=0.2.12`, `shacl_ir=0.2.9`, `prefixmap=0.2.9`, `mie=0.2.9`), `serde_json`, `serde-wasm-bindgen`. Bump `wasm-bindgen` `0.2.100`→`=0.2.108`.
- [ ] T002 Bump the `wasm-bindgen-cli` pin in `flake.nix`/`nix/packages.nix` `0_2_100`→`0_2_108` in the SAME commit as T001 (Constitution II: lib == cli).
- [ ] T003 Reconcile `getrandom` for the wasm build in `nix/crane.nix`/`nix/packages.nix`: one config satisfying both oxigraph (`js`) and rudof (`wasm_js` + `--cfg getrandom_backend="wasm_js"`). Confirm `cargo check -p rdf-shapes-wasm --target wasm32-unknown-unknown` builds.
- [ ] T004 [P] Widen `deny.toml` license allow-list for the oxigraph/rudof trees (run `nix build .#checks.<sys>.deny`, add only the licenses actually encountered).
- [ ] T005 Verify `nix flake check` still green after the dependency churn (no new logic yet).

**Checkpoint**: workspace builds native + wasm32 with both engines linked.

---

## Phase 2: Foundational (shared types — BLOCKS US1/US2)

- [ ] T006 Create `crates/rdf-shapes-core/src/error.rs`: a `thiserror` enum `ShapesError` with `Parse`, `Query`, `Validation`, `Unsupported(&'static str)` variants; re-export from `lib.rs`.
- [ ] T007 [P] Create the result/report serde types in `crates/rdf-shapes-core/src/`: `QueryResults` (Select rows / Ask bool / Graph) and `ValidationReport { conforms, violations: Vec<Violation{ focus_node, path?, value?, source_constraint_component, message, severity }> }`.

**Checkpoint**: error model + output structs exist and compile (native + wasm32).

---

## Phase 3: User Story 1 — SPARQL query (P1) 🎯 MVP

**Independent test**: run a real treasury named query over a sample graph → expected rows.

- [ ] T008 [US1] Unit test in `crates/rdf-shapes-core/src/query.rs` (`#[cfg(test)]`): load sample Turtle, run `tx-count`/`asset-flow` SELECT, assert rows; an ASK test; a malformed-query test asserting `ShapesError::Query`. Watch it FAIL.
- [ ] T009 [US1] Implement `query(graph_ttl, sparql) -> Result<QueryResults, ShapesError>` in `query.rs`: oxigraph in-memory `Store`, load Turtle, evaluate, serialize via Oxigraph's **SPARQL Query Results JSON** writer (typed terms, not lexical `to_string`).
- [ ] T010 [US1] Re-export `query` from `rdf-shapes-core/src/lib.rs`; remove/retire the scaffold `ping` placeholder.
- [ ] T011 [US1] Wasm shim: `#[wasm_bindgen] pub fn query(...)` in `crates/rdf-shapes-wasm/src/lib.rs` via `serde-wasm-bindgen`; errors → `Err(JsValue)`.
- [ ] T012 [US1] CLI: `rdf-shapes query --graph <ttl> --query <rq>` subcommand in `crates/rdf-shapes-cli/src/main.rs` (clap), printing JSON; non-zero exit on error.

**Checkpoint**: SPARQL works in core (native test), wasm, and CLI.

---

## Phase 4: User Story 2 — SHACL Core validation (P1)

**Independent test**: validate `history-entry.shacl.ttl` against conforming + violating graphs.

- [ ] T013 [US2] Unit test in `crates/rdf-shapes-core/src/validate.rs` (`#[cfg(test)]`): copy the real `history-entry.shacl.ttl` into `crates/rdf-shapes-core/testdata/`; conforming graph → `conforms`; violating graph → violations for `sh:minCount`/`sh:datatype`/`sh:nodeKind` with correct components. Watch it FAIL.
- [ ] T014 [US2] Implement `validate(data_ttl, shapes_ttl) -> Result<ValidationReport, ShapesError>` in `validate.rs`: rudof `shacl_validation` (Core, in-memory), map to `ValidationReport`; SHACL-SPARQL/remote → `ShapesError::Unsupported`.
- [ ] T015 [US2] Re-export `validate` from `lib.rs`.
- [ ] T016 [US2] Wasm shim `#[wasm_bindgen] pub fn validate(...)` in `rdf-shapes-wasm`.
- [ ] T017 [US2] CLI `rdf-shapes validate --data <ttl> --shapes <ttl>` subcommand.

**Checkpoint**: SHACL Core works in core (native test), wasm, and CLI.

---

## Phase 5: User Story 3 — portable + reproducible artifact (P2)

**Independent test**: same inputs → equivalent wasm vs CLI output; double-build SHA identical.

- [ ] T018 [US3] Update `nix/packages.nix` so `wasm-pkg` carries the real engines; confirm `nix build .#wasm-pkg` and record the size (budget note if >~5 MB).
- [ ] T019 [US3] Reproducibility check: `nix build .#wasm-pkg` then `--rebuild`; assert identical `.wasm` SHA-256 (document in PR).
- [ ] T020 [P] [US3] Parity smoke: a Node wasm smoke + a CLI run on the same graph+query and graph+shapes; assert equivalent output. Add a `spike`-style README under `crates/rdf-shapes-wasm/smoke/` (gitignore the built bundle).

**Checkpoint**: SC-003 (browser/CLI parity) and SC-004 (reproducible) demonstrated.

---

## Phase N: Polish & docs

- [ ] T021 [P] Update MkDocs `docs/usage.md` (real `query`/`validate` examples) and confirm rustdoc still renders under `/api/`.
- [ ] T022 `nix flake check` fully green (clippy `-D warnings`, fmt, nextest, deny, doc); `just ci` green.
- [ ] T023 Update the PR body as a tour; confirm CI's 5 required checks pass.

---

## Dependencies & order

- Phase 1 → Phase 2 → (US1 ∥ US2) → US3 → Polish.
- US1 and US2 are independent (different files: `query.rs` vs `validate.rs`) and may proceed in parallel after Phase 2; each is independently testable/demoable.
- Within a story: test (fails) → core impl → lib re-export → wasm shim → CLI.
- Commit per task or logical group; one PR (#6) with reviewer-curated slices.

## Notes

- Conformance vs W3C suites + Jena differential is **#7**, not here (SC-001/SC-002 reference it).
- The rudof family `=` pins (T001) are mandatory — the 0.2.x tree breaks across the `iri_s`→`rudof_iri` rename (spike #4 finding).
