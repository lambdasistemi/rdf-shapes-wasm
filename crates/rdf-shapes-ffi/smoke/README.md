# Native Haskell ccall smoke

The live-boundary proof for [#19](https://github.com/lambdasistemi/rdf-shapes-wasm/issues/19):
the Haskell backend consumes the engine through the native C-ABI shared
library, not the wasm. Spike #5 established that native
`foreign import ccall` works from **GHC 9.12.3** (the `wasmtime-hs` path
is blocked on 9.12.3); this smoke is that path made real against the
production `cdylib`.

[`Main.hs`](Main.hs) binds the four exported symbols
(`rdf_shapes_query`, `rdf_shapes_validate`, `rdf_shapes_version`,
`rdf_shapes_string_free`) with `foreign import ccall`, marshals C
strings across the boundary, frees every returned envelope, and asserts
the engine actually answered (3 transactions counted, the conforming
graph conforms, a malformed query surfaces an `{"error":...}`).

Run it (only prerequisite is Nix — it builds `.#ffi-lib` and pulls GHC
9.12.3 from nixpkgs):

```bash
crates/rdf-shapes-ffi/smoke/run-smoke.sh
```

Expected output:

```text
version : {"ok":"0.1.0"}
query   : {"ok":{"json":{"head":{"vars":["transactions"]},"results":{"bindings":[{"transactions":{"datatype":"http://www.w3.org/2001/XMLSchema#integer","type":"literal","value":"3"}}]}},"kind":"solutions"}}
validate: {"ok":{"conforms":true,"violations":[]}}
error   : {"error":"query error: parse: error at 1:10: expected CONSTRUCT"}
SMOKE OK: Haskell ccall reached the native engine
```

This is a named operator follow-up, not part of `nix flake check`:
running it pulls a ~280 MiB GHC closure, which doesn't belong in the
per-PR gate. The crate's own behaviour is gated by the `#[cfg(test)]`
Rust round-trip tests under `nextest`.
