# Changelog

## 0.1.0

The first release. One Rust source (`rdf-shapes-core`), shipped to
multiple targets — browser wasm, native FFI lib, and native CLI.

- **Engine** — full **SPARQL 1.1** query (Oxigraph) and **SHACL Core**
  validation (rudof) over in-memory Turtle, with deterministic,
  JSON-faithful output.
- **Reproducible wasm artifact** — an npm-shaped `wasm-bindgen` bundle
  built by Nix from the committed `Cargo.lock`; two clean builds yield a
  byte-identical `.wasm`, and `SHA256SUMS` ships with each release.
- **Native FFI library** — a C-ABI shared library
  (`librdf_shapes_ffi.{so,dylib}`) plus a cbindgen-generated header, the
  server reuse contract; consumed from Haskell via `foreign import
  ccall` (verified on GHC 9.12.3).
- **Native CLI** — the self-contained `rdf-shapes` binary (`query` /
  `validate`) for CI and scripts.
- **Browser playground** — a PureScript/Halogen client-side app over the
  same wasm engine, live at
  <https://lambdasistemi.github.io/rdf-shapes-wasm/app/>.
- **Documentation site** — the MkDocs Material site (concepts,
  architecture, usage, reuse) with co-hosted rustdoc, live at
  <https://lambdasistemi.github.io/rdf-shapes-wasm/>.
