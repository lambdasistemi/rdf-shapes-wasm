# Tasks: Browser SPARQL+SHACL playground

**Input**: [spec.md](./spec.md), [plan.md](./plan.md) · **Issue**: #16
**Tests**: INCLUDED for pure logic (permalink/machine-path/examples) + a browser
smoke. Engine correctness is covered by #6/#7.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup (toolchain + scaffold)

- [ ] T001 Add `purescript-overlay` + `mkSpagoDerivation` flake inputs to `flake.nix` (per the `purescript` skill); add `nodejs_20`, `purs`, `spago-unstable`, `purs-tidy-0_10_0`, `esbuild` to the dev shell.
- [ ] T002 Scaffold `app/`: `spago.yaml` (halogen + deps, registry pin 72.1.0), `package.json`, `dist/index.html`, `src/Main.purs` (Halogen runUI placeholder). Generate + commit `spago.lock` and `package-lock.json`.
- [ ] T003 `nix/playground.nix`: `mkSpagoDerivation` build; copy the `wasm-pkg` derivation output (`rdf_shapes_wasm.js`, `_bg.wasm`) into `app/vendor/` before bundling; esbuild `bootstrap.js` (with `--loader:.wasm=binary`) then `spago bundle`, concatenated. Wire `packages.playground` + `checks.playground` (purs-tidy check + `spago build`/`test`) in `flake.nix`.
- [ ] T004 Confirm `nix build .#playground` produces `dist/{index.html,index.js}` and `nix flake check` includes + passes the playground check.

**Checkpoint**: empty Halogen app builds reproducibly through Nix and loads the wasm engine.

---

## Phase 2: Engine FFI (blocking US1)

- [ ] T005 `src/bootstrap.js`: import the vendored `wasm-pkg` ESM + `_bg.wasm`, `initSync`, expose `globalThis.rdfShapes`.
- [ ] T006 `src/FFI/RdfShapes.{purs,js}`: `query :: String -> String -> Effect (Either String Json)` and `validate :: String -> String -> Effect (Either String Json)` over `globalThis.rdfShapes` (catch thrown `JsValue` → `Left`).

**Checkpoint**: PureScript can call query/validate and get JSON or an error.

---

## Phase 3: User Story 1 — combined run view (P1) 🎯 MVP

- [ ] T007 [US1] `src/Playground.purs`: Halogen component with three inputs (TTL data, SPARQL, SHACL shapes), a Run action, and two result regions (query results, validation report); copyable (pretty-printed JSON), per-pane error display.
- [ ] T008 [US1] Wire `Main.purs` to mount `Playground`; basic styling in `dist/index.html` (Material-ish defaults, no bespoke CSS framework — keep minimal).
- [ ] T009 [US1] Browser smoke: serve `dist/`, load the page, paste a sample graph + `tx-count` + the `history-entry` shape, Run, assert results + report render (playwright MCP if available; otherwise document a manual curl+JS check). Confirms FR-001/002/003 end to end.

**Checkpoint**: the core playground works in a real browser.

---

## Phase 4: User Story 2 — preloaded examples (P2)

- [ ] T010 [US2] `src/Examples.purs`: 2–3 example `Session`s (treasury-shaped graph + a named query; `history-entry` shape + conforming and violating data). Lift the sample TTL/queries from the engine testdata.
- [ ] T011 [US2] Add an example picker to `Playground`; selecting one fills the inputs. Unit test that each example's data is non-empty/well-formed.

---

## Phase 5: User Story 3 — shareable permalinks (P2)

- [ ] T012 [US3] `src/Permalink.purs`: pure `encode :: Session -> String` / `decode :: String -> Maybe Session` via JSON→base64url→`#s=`. **Unit test the round-trip** (Test.Main, spec).
- [ ] T013 [US3] `src/FFI/Location.{purs,js}`: read the URL (hash + query), and replace the hash without reload. On init, decode `#s=` and populate; add a "Copy link" action that encodes current inputs into the hash.

---

## Phase 6: User Story 4 — URL-param machine path (P3)

- [ ] T014 [US4] On init, if `?ttl=&sparql=&shapes=` present (and no `#s=`), populate AND auto-run, rendering results prominently. Pure param→Session parsing unit-tested. Document the contract in `app/README.md`.

---

## Phase 7: Deploy + docs

- [ ] T015 Extend `.github/workflows/deploy-docs.yml`: build `nix build .#playground`, copy its `dist/*` into `site/app/`, so Pages serves the playground at `/app/`. Link it from the MkDocs nav + `docs/index.md`.
- [ ] T016 [P] `app/README.md`: what it is, the URL, the permalink + URL-param contracts, how to run locally (`just dev`).
- [ ] T017 `nix flake check` fully green (Rust checks + the new playground check); `just ci` green; CI's required checks pass on the PR.

---

## Dependencies & order

- Phase 1 → 2 → US1 (MVP) → US2 ∥ US3 → US4 → deploy.
- Permalink/examples/machine-path pure logic is unit-tested (spec); the browser smoke (T009) covers the end-to-end engine wiring.
- One PR (#16), reviewer-curated slices, Conventional Commits.

## Notes

- The app adds NO evaluation logic (Constitution I) — all SPARQL/SHACL goes
  through the `wasm-pkg`.
- Deploy is the SINGLE repo Pages site; extend `deploy-docs.yml`, do not add a
  second Pages workflow (they would conflict).
- Keep UI minimal/standard — no bespoke CSS framework (org "don't invent UI").
