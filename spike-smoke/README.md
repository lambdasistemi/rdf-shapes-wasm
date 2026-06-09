# SPIKE (#3): SPARQL 1.1 in wasm via Oxigraph

Throwaway de-risking spike for epic #1. Proves Oxigraph's in-memory
store compiles for `wasm32-unknown-unknown` and runs a SPARQL 1.1
SELECT in a headless JS runtime.

`crates/rdf-shapes-core` gained a `query(turtle, sparql) -> json`
function backed by an in-memory `oxigraph::store::Store`;
`crates/rdf-shapes-wasm` re-exports it over `#[wasm_bindgen]`.

## Verdict (the deliverable)

- `oxigraph = "0.5.8"`, **`default-features = false`** (drops the
  `rocksdb` default that won't build for wasm32), **`features = ["js"]`**
  (wires `oxsdatatypes/js` + `js-sys` + `getrandom/wasm_js`).
- getrandom: `0.3.4`, backend selected purely by oxigraph's `js`
  feature — **no `RUSTFLAGS=--cfg getrandom_backend="wasm_js"` needed**,
  `cargo check --target wasm32-unknown-unknown` is green as-is.
- Forced a `wasm-bindgen` bump `0.2.100 -> 0.2.108` workspace-wide
  (js-sys 0.3.85 requires it) and the `wasm-bindgen-cli` pin in
  `flake.nix` in lockstep.
- Sizes: raw cdylib 2.7 MB; after `wasm-bindgen --target nodejs` 2.3 MB;
  after `wasm-opt -Oz` ~2.1 MB. wasm build ~31 s.
- SELECT result shape: JSON array of row objects; each binding is the
  SPARQL term lexical form, so typed literals carry the datatype suffix,
  e.g. `"3"^^<http://www.w3.org/2001/XMLSchema#integer>`.

## Run the headless smoke

From the repo root, inside the Nix dev shell (provides cargo, the
pinned `wasm-bindgen-cli`, `wasm-opt`, and node):

```sh
nix develop -c bash -c '
  set -e
  cargo build --release -p rdf-shapes-wasm --target wasm32-unknown-unknown
  mkdir -p spike-smoke/pkg
  wasm-bindgen --target nodejs \
    --out-dir spike-smoke/pkg --out-name rdf_shapes_wasm \
    target/wasm32-unknown-unknown/release/rdf_shapes_wasm.wasm
  wasm-opt -Oz --enable-bulk-memory --enable-mutable-globals \
    -o spike-smoke/pkg/rdf_shapes_wasm_bg.wasm \
    spike-smoke/pkg/rdf_shapes_wasm_bg.wasm
  node spike-smoke/smoke.mjs
'
```

Expected output ends with `SMOKE OK: tx-count = 3`.

## Files

- `sample.ttl` — small hand-crafted graph (3 `cardano:Transaction`).
- `tx-count.rq` — copied verbatim from the treasury named queries
  (`amaru-treasury-tx/lib/Amaru/Treasury/History/queries/tx-count.rq`).
- `smoke.mjs` — Node runner: loads the bundle, calls `query()`, asserts.
- `pkg/` — generated bundle (gitignored; rebuild with the command above).
