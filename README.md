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

## License

[Apache-2.0](./LICENSE)
