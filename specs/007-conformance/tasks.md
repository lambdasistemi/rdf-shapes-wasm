# Tasks: Conformance + Jena differential harness

**Input**: [spec.md](./spec.md), [plan.md](./plan.md) · **Issue**: #7
**Gate**: `nix flake check` (adds `conformance`). Apache Jena (`apache-jena`,
nixpkgs 5.x) is the oracle.

## Phase 1: Corpus (shared)

- [ ] T001 Create `conformance/corpus/sparql/<case>/{graph.ttl,query.rq}` for the treasury named queries (`tx-count`, `asset-flow`, `spend-edges`, `entity-occurrences`, `history-entries`, `address-resolution`) — copy the `.rq` from `/code/amaru-treasury-tx/lib/Amaru/Treasury/History/queries/` and craft small sample graphs that exercise each (note provenance in `conformance/README.md`).
- [ ] T002 Create `conformance/corpus/shacl/<case>/{data.ttl,shapes.ttl}` for `history-entry` + `indexed-tx-body` shapes (copy from `.../History/shapes/`) with conforming and violating data.
- [ ] T003 Add a few extra plain SPARQL-1.1 (SELECT/ASK/CONSTRUCT, ORDER BY, aggregates) and SHACL-Core (minCount/datatype/nodeKind/class) cases to broaden coverage. `conformance/README.md`: layout + how to add a case + how to regen.

**Checkpoint**: a self-contained corpus exists.

## Phase 2: Harness crate (blocks US1/US2)

- [ ] T004 New native crate `crates/rdf-shapes-conformance` (bin). Deps: `rdf-shapes-core`, `serde_json`, Turtle/N-Triples parsing (oxttl/oxrdf or core helpers). Not required to build for wasm (dev/test tool; see plan Complexity Tracking) — keep it out of the wasm build.
- [ ] T005 [US1] Comparison module: SELECT multiset eq (term kind/value/datatype/lang; **blank-node structural canonicalization**, order only with `ORDER BY`); ASK bool; CONSTRUCT/DESCRIBE canonical triple-set; SHACL `conforms` + violation set keyed by (focus, component, path). Unit-test the comparators (incl. order-insensitive + blank-node cases). Watch them fail first.
- [ ] T006 Engine side: run each corpus item through `rdf-shapes-core` `query`/`validate`; normalize to the comparison form.

**Checkpoint**: comparators + engine-side run, unit-tested.

## Phase 3: User Story 1 — differential vs Jena (P1) 🎯

- [ ] T007 [US1] Jena side: shell to `arq --data … --query … --results=json` and `shacl validate --shapes … --data …`; parse `arq` SRJ and the `shacl` report graph into the comparison form. Jena found on `PATH`.
- [ ] T008 [US1] Differential driver: iterate `conformance/corpus/{sparql,shacl}`, run engine vs Jena, compare, collect divergences; non-zero exit + per-item report on any divergence; print pass/skip summary.

**Checkpoint**: `rdf-shapes-conformance` passes engine-vs-Jena on the seed corpus (or surfaces a documented divergence).

## Phase 4: User Story 2 — W3C conformance (P2)

- [ ] T009 [US2] Curate a representative set of W3C **SPARQL 1.1 query** + **SHACL Core** cases into `conformance/corpus/w3c/...` with their official **expected** results (record source repo + commit in README). SHACL-SPARQL/federation/remote → explicit skip entries with reasons.
- [ ] T010 [US2] "expected" mode in the harness: compare engine output to the committed expected result (same comparators); emit pass/skip counts; skips are explicit + reasoned (no silent pass/fail).

## Phase 5: User Story 3 — wire the gate (P2)

- [ ] T011 [US3] `nix/conformance.nix`: a `runCommand` `checks.conformance` (per the `/nix` skill — real sandboxed check) with `apache-jena` + the crane-built harness on PATH, running it over `${src}/conformance`. Wire `packages.conformance` (the runner) + `checks.conformance` into `flake.nix`; add a CI `conformance` job (+ Build Gate warm step) and `just conformance` / `nix run .#conformance`.
- [ ] T012 [US3] Negative test: deliberately perturb one corpus expected/case and confirm `checks.conformance` FAILS (the gate is real); then revert. Document it in the PR.

## Phase 6: Polish

- [ ] T013 [P] Update `docs/architecture.md` (or a `docs/conformance.md`) describing the trust model: W3C + Jena-differential as the arbiter; link from the README. Note any documented divergences.
- [ ] T014 `nix flake check` fully green (existing checks + `conformance`); `just ci` green; CI required-check set updated if appropriate.

## Dependencies & order

- Phase 1 → 2 → US1 → US2 → gate → polish.
- One PR (#7), Conventional commits, reviewer-curated slices.

## Notes

- "Parity" = **semantic equivalence on normalized output** (engines format
  reports differently), per spec Assumptions.
- A real engine↔Jena divergence is a **finding to triage in the PR**, not
  something to silence — it's the whole point of the harness.
- Full manifest-driven W3C ingestion is a future expansion; this delivers a
  curated, extensible set.
