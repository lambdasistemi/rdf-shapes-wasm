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
| `rdf-shapes-wasm` | `cdylib` + `rlib`. Thin `#[wasm_bindgen]` shims over the core. |
| `rdf-shapes-cli` | Native `rdf-shapes` binary. Thin `clap` front end over the core. |

The current surface is intentionally trivial (`version` / `ping`); the SPARQL
and SHACL engines arrive with later spikes.

## Commands

Everything runs through Nix, so the gate is identical locally and in CI.

| Command | What it does |
|---|---|
| `nix flake check` | The single gate: clippy (`-D warnings`), rustfmt, nextest, cargo-deny, rustdoc (`-D warnings`). |
| `just ci` | Same gate via `nix run .#ci` (builds every package + every check). |
| `nix build .#cli` | Native CLI; run `./result/bin/rdf-shapes --help`. |
| `nix build .#wasm-pkg` | Reproducible, npm-shaped wasm bundle (pinned `wasm-bindgen-cli` + `wasm-opt -Oz`). |
| `nix build .#release-artifacts` | CLI tarball + npm `.tgz` + bare `.wasm` + `SHA256SUMS`. |

The `.wasm` is reproducible: two clean builds of `.#wasm-pkg` yield a
byte-identical artifact. The `wasm-bindgen` library version is locked to the
pinned `wasm-bindgen-cli` (see `Cargo.toml` and `flake.nix`).

## License

[Apache-2.0](./LICENSE)
