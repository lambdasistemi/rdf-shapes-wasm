# shacl-spike — THROWAWAY: SHACL Core in wasm (issue #4)

De-risking spike for epic #1's linchpin question: **can SHACL Core
validation run in `wasm32-unknown-unknown`?**

**Verdict: YES.** Via the rudof project's `shacl_validation` crate,
SHACL Core (node/property shapes, cardinality, datatype, nodeKind,
class, value-range, …) validates entirely in-memory in wasm. No fs, no
network, no SPARQL endpoint, no threads. See issue #4 for the full
write-up.

This crate is **reference code**, not production. It is intentionally
outside the `rdf-shapes-core` / `-wasm` / `-cli` layout and does not opt
into the workspace lint surface (spikes are exempt per the
constitution).

## What it does

Exposes one wasm-bindgen function:

```
validate(data_ttl: string, shapes_ttl: string) -> string  // JSON Report
```

`Report = { conforms, violation_count, violations[], error }`. Both
inputs are Turtle source held in memory.

## Crate selection / version pins

`shacl_validation = "0.2.12"` with `default-features = false` (drops the
`sparql` feature, which is the only thing that pulls
`reqwest`/`tokio`/`spargebra` — none of which build for wasm). The Native
(in-memory) engine handles SHACL Core.

The rudof 0.2.x tree is **internally inconsistent**: `shacl_ast/_ir/_rdf`
top out at `0.2.9` (still using `iri_s` + `prefixmap ^0.2.3`), but at
`rudof_rdf >= 0.2.14` rudof renamed `iri_s -> rudof_iri` and bumped
`prefixmap`/`mie` to 0.2.20 — which no longer type-checks against
`shacl_ast 0.2.9`. We therefore pin the whole family to the last
`iri_s`-era, wasm-capable releases:

| crate            | pin       | why                                         |
|------------------|-----------|---------------------------------------------|
| `shacl_validation` | `0.2.12`  | latest; Native engine; `default-features=false` |
| `rudof_rdf`      | `=0.2.12` | last `iri_s` release; first with `reqwest`/`tokio` optional + wasm cfg-gating |
| `sparql_service` | `=0.2.12` | non-optional dep of `shacl_validation`; held pre-rename |
| `shacl_ir`       | `=0.2.9`  | latest; uses `iri_s`                        |
| `prefixmap`      | `=0.2.9`  | pre-`rudof_iri`                             |
| `mie`            | `=0.2.9`  | pre-`rudof_iri`                             |

`getrandom 0.3` reaches the tree transitively; on wasm it needs the JS
backend, enabled with the `wasm_js` feature **and** the build cfg
`--cfg getrandom_backend="wasm_js"`.

## Build + run the headless smoke

From the repo root, inside `nix develop`:

```bash
# 1. compile the cdylib for wasm32 (note the getrandom cfg)
export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'
cargo build -p shacl-spike --target wasm32-unknown-unknown --release

# 2. generate the nodejs bindings
wasm-bindgen --target nodejs \
  --out-dir crates/shacl-spike/smoke/pkg \
  target/wasm32-unknown-unknown/release/shacl_spike.wasm

# 3. (optional) finalize — wasm-opt needs these flags on recent rustc
wasm-opt -Oz --enable-bulk-memory --enable-mutable-globals \
  crates/shacl-spike/smoke/pkg/shacl_spike_bg.wasm \
  -o crates/shacl-spike/smoke/pkg/shacl_spike_bg.wasm

# 4. run the smoke (validates the real treasury shape)
node crates/shacl-spike/smoke/smoke.mjs
```

Expected: the conforming graph reports `conforms:true`; the violating
graph reports `conforms:false` with three violations (`sh:nodeKind`,
`sh:minCount`, `sh:datatype`).

Native logic is also exercised by `cargo test -p shacl-spike`.

## Test data

- `testdata/history-entry.shacl.ttl` — copied verbatim from
  `amaru-treasury-tx`.
- `testdata/conforming.ttl` — a well-formed `atx:HistoryEntry`.
- `testdata/violating.ttl` — breaks nodeKind + minCount + datatype.
