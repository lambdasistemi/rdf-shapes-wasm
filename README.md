# rdf-shapes-wasm

Portable **SPARQL 1.1** query + **SHACL Core** validation engine, compiled to
**WebAssembly**.

One Rust-built `.wasm` artifact runs in the browser, on the server (via a wasm
host), in CI, and as a self-contained release blob — replacing JVM/Apache-Jena
CLI dependencies for querying and validating RDF transaction graphs.

## Engines

- [Oxigraph](https://github.com/oxigraph/oxigraph) — SPARQL 1.1 query
- [rudof](https://github.com/rudof-project/rudof) — SHACL Core validation

## Status

Early scaffolding. The roadmap is tracked in the
[issues](https://github.com/lambdasistemi/rdf-shapes-wasm/issues).

Built and tested entirely with Nix — `nix flake check` / `just ci`. Rust
toolchain via [crane](https://github.com/ipetkov/crane) + rust-overlay.

## Workspace layout

| Crate | Role |
|---|---|
| `rdf-shapes-core` | Portable core logic. Builds native **and** `wasm32`; no `wasm-bindgen`, no I/O. All logic lives here. |
| `rdf-shapes-wasm` | `cdylib` + `rlib`. Thin `#[wasm_bindgen]` shims over the core (browser). |
| `rdf-shapes-ffi` | `cdylib`. Thin `extern "C"` shims over the core (native server / Haskell). |
| `rdf-shapes-cli` | Native `rdf-shapes` binary. Thin `clap` front end over the core. |

The core exposes `query` (full SPARQL 1.1, via Oxigraph) and `validate`
(SHACL Core, via rudof) over in-memory Turtle; the wasm, FFI, and CLI
shells marshal those to and from JSON. See [`docs/usage.md`](docs/usage.md).

`rdf-shapes-ffi` is the native sibling of `rdf-shapes-wasm`: where the
wasm crate hands the engine to the browser, the FFI crate hands the same
engine to a native host (the Haskell backend, via `foreign import
ccall`) over a C-string ABI. Each function returns a JSON envelope —
`{"ok": <result>}` on success, `{"error": "<message>"}` on failure — and
the caller frees the returned string with `rdf_shapes_string_free`. See
[`crates/rdf-shapes-ffi/smoke`](crates/rdf-shapes-ffi/smoke) for the GHC
9.12.3 ccall proof.

## Commands

Everything runs through Nix, so the gate is identical locally and in CI.

| Command | What it does |
|---|---|
| `nix flake check` | The single gate: clippy (`-D warnings`), rustfmt, nextest, cargo-deny, rustdoc (`-D warnings`). |
| `just ci` | Same gate via `nix run .#ci` (builds every package + every check). |
| `nix build .#cli` | Native CLI; run `./result/bin/rdf-shapes --help`. |
| `nix build .#wasm-pkg` | Reproducible, npm-shaped wasm bundle (pinned `wasm-bindgen-cli` + `wasm-opt -Oz`). |
| `nix build .#ffi-lib` | Native C-ABI shared library: `lib/librdf_shapes_ffi.{so,dylib}` + `include/rdf_shapes.h` (cbindgen-generated). |
| `nix build .#release-artifacts` | CLI tarball + npm `.tgz` + bare `.wasm` + `SHA256SUMS`. |

The `.wasm` is reproducible: two clean builds of `.#wasm-pkg` yield a
byte-identical artifact. The `wasm-bindgen` library version is locked to the
pinned `wasm-bindgen-cli` (see `Cargo.toml` and `flake.nix`).

## License

[Apache-2.0](./LICENSE)
