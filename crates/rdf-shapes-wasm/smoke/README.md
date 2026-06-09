# wasm ↔ CLI parity smoke

Proves User Story 3 / SC-003: the **same** SPARQL query and the **same**
SHACL validation produce equivalent output through the browser-shaped
wasm bundle and the native CLI.

`run-smoke.sh` builds the reproducible `wasm-pkg`, links it under
`pkg/`, builds the native CLI, runs both on the shared inputs, and
diffs the canonical JSON. It exits non-zero on any mismatch.

## Run

From the repo root, inside the Nix dev shell (provides `node`, the
pinned `wasm-bindgen-cli`, `wasm-opt`, and `cargo`):

```sh
nix develop -c crates/rdf-shapes-wasm/smoke/run-smoke.sh
```

Expected tail: `SMOKE OK: wasm and CLI agree on query and validate`.

## Inputs

- `graph.ttl` + `tx-count.rq` — three `cardano:Transaction` nodes and
  the real treasury `tx-count` named query (a typed SELECT count).
- `data.ttl` + `shapes.ttl` — the violating history entry and the real
  `history-entry.shacl.ttl` (a non-conforming report with three
  SHACL Core violations).

## Files

- `smoke.mjs` — Node runner: instantiates the `--target web` bundle
  from the `.wasm` bytes, calls `query()` and `validate()`, prints
  canonical JSON.
- `run-smoke.sh` — builds + links + runs both surfaces and diffs.
- `pkg/` — the generated bundle (gitignored; `run-smoke.sh` rebuilds it).
