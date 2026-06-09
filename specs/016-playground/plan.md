# Implementation Plan: Browser SPARQL+SHACL playground

**Branch**: `feat/playground` | **Date**: 2026-06-09 | **Spec**: [spec.md](./spec.md)
**Issue**: #16 (epic #1)

## Summary

A PureScript/Halogen single-page app that consumes the `rdf-shapes-wasm`
`wasm-pkg` (#6) to run SPARQL 1.1 and SHACL Core over pasted Turtle, fully
client-side. Combined run view + preloaded examples + shareable permalinks +
URL-param machine path. Built with spago + esbuild, packaged via Nix, and served
from the repo's GitHub Pages under `/app/`.

## Technical Context

**Language/Version**: PureScript (`purs`), `spago-unstable` (Spago 2), `purs-tidy` 0.10.0
**Primary Dependencies**: `halogen`, `aff`, `web-html`, `argonaut`/`json` for result rendering; npm: the Nix-built `wasm-pkg` (no registry publish)
**Build**: `mkSpagoDerivation` + `purescript-overlay` (the `purescript` skill); esbuild bundles `bootstrap.js` (npm/wasm) then `spago bundle` (PS), concatenated; `--loader:.wasm=binary`
**Engine integration**: `bootstrap.js` imports the `wasm-pkg` ESM + its `_bg.wasm`, calls `initSync`, exposes `globalThis.rdfShapes`; a `RdfShapes` FFI module calls `query`/`validate`
**Target Platform**: modern browsers (WebAssembly + ES modules)
**Project Type**: web app (PureScript) consuming a wasm library — lives in `app/`
**Performance Goals**: sub-second evaluation on a typical treasury graph (engine SC-005)
**Constraints**: fully client-side / offline after load; reproducible Nix build; deployed under the single repo Pages site
**Scale/Scope**: single-page app, three text inputs, two result views, ~handful of Halogen components

## Constitution Check

- [x] **I. Pure Portable Core, Thin Shells** — the app adds NO evaluation logic;
  it is a thin UI shell over the `wasm-pkg`. (`rdf-shapes-core` stays the only
  place logic lives.)
- [x] **II. Reproducible, Content-Addressed Artifacts** — built by
  `mkSpagoDerivation` from committed `spago.lock` + `package-lock.json`,
  consuming the Nix-built `wasm-pkg` derivation (no `npm install` at build, no
  network); the bundle is content-addressed by its inputs.
- [x] **III. Conformance & Differential Correctness** — the app delegates all
  correctness to the engine (covered by #6 tests and #7 conformance); the app's
  own tests cover its pure logic (permalink encode/decode, URL-param parsing,
  example data) plus a browser smoke that a query+validate actually run.
- [x] **IV. Nix-First Single Gate** — a `playground` package + a `playground`
  check (purs-tidy `check` + `spago build`/`test`) join `nix flake check` ==
  `just ci` == CI. The PureScript app uses `purs-tidy`, not clippy/rustfmt, but
  the single-gate principle holds.
- [x] **V. Disciplined Delivery** — issue-backed PR (#16), linear history,
  Conventional Commits, committed lockfiles.

No violations → Complexity Tracking empty.

## Project Structure

```text
app/                         # the PureScript playground (new)
├── spago.yaml               # halogen + deps; registry pin 72.1.0
├── spago.lock               # committed
├── package.json             # npm: (wasm-pkg vendored) ; committed package-lock.json
├── dist/index.html          # static shell
└── src/
    ├── bootstrap.js         # imports wasm-pkg ESM + _bg.wasm -> initSync -> globalThis.rdfShapes
    ├── Main.purs            # Halogen runUI; reads URL on init (permalink + machine path)
    ├── FFI/RdfShapes.purs   # query / validate bindings
    ├── FFI/RdfShapes.js     # globalThis.rdfShapes.{query,validate}
    ├── FFI/Location.purs    # read/replace URL (hash + query params)
    ├── FFI/Location.js
    ├── Playground.purs      # the combined-view Halogen component
    ├── Examples.purs        # preloaded example sessions
    └── Permalink.purs       # encode/decode Session <-> URL (pure; unit-tested)
nix/
└── playground.nix           # mkSpagoDerivation build, fed the wasm-pkg output
flake.nix                    # + purescript-overlay & mkSpagoDerivation inputs; packages.playground; checks.playground
.github/workflows/deploy-docs.yml  # also build playground -> copy into site/app/
```

**Structure Decision**: Keep the app isolated under `app/` so the Rust workspace
and the PureScript app don't entangle; the flake gains the PureScript inputs and
two outputs (`packages.playground`, `checks.playground`).

### Engine integration detail

The `wasm-pkg` derivation emits `rdf_shapes_wasm.js` + `rdf_shapes_wasm_bg.wasm`
(`--target web`). `nix/playground.nix` copies those into the app source tree
(e.g. `app/vendor/`) before bundling; `bootstrap.js` does
`import * as rdfShapes from "./vendor/rdf_shapes_wasm.js"` +
`import wasmBytes from "./vendor/rdf_shapes_wasm_bg.wasm"` +
`rdfShapes.initSync({ module: new WebAssembly.Module(wasmBytes) })` +
`globalThis.rdfShapes = rdfShapes`. (Mirrors the `graph-browser` Oxigraph
pattern from the `purescript` skill.)

### Permalink & machine path

- **Permalink**: `Session` → JSON → base64url → URL hash (`#s=…`). On load, if a
  hash is present, decode and populate. Pure `encode`/`decode` in `Permalink.purs`
  (unit-tested round-trip).
- **Machine path**: query params `?ttl=&sparql=&shapes=` (URL-encoded). On load,
  if present, populate AND auto-run, rendering results prominently. Hash permalink
  takes precedence if both present.

## Complexity Tracking

No constitution violations — table intentionally empty.
