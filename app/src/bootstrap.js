// esbuild entry point: load the vendored rdf-shapes-wasm engine and
// expose its initialized exports on globalThis for the PureScript FFI.
//
// The vendored files come from the Nix `wasm-pkg` derivation
// (`--target web`), copied into `app/vendor/` by `nix/playground.nix`
// before bundling. The `.wasm` is inlined as binary via esbuild's
// `--loader:.wasm=binary`, so there is no runtime fetch — evaluation is
// fully client-side (FR-002 / SC-005).
import * as rdfShapes from "./vendor/rdf_shapes_wasm.js";
import wasmBytes from "./vendor/rdf_shapes_wasm_bg.wasm";

// Synchronous init from the inlined module bytes. `initSync` accepts a
// `{ module }` object; the `#[wasm_bindgen(start)]` panic hook runs
// automatically during finalization.
rdfShapes.initSync({ module: new WebAssembly.Module(wasmBytes) });

globalThis.rdfShapes = rdfShapes;
