# rdf-shapes-wasm

Portable **SPARQL 1.1** query + **SHACL Core** validation engine, written once
in Rust (`rdf-shapes-core`) and shipped to **multiple targets from one source**:

- **browser wasm** — a reproducible `wasm-bindgen` bundle (client-side, no
  server),
- **native FFI lib** — a C-ABI shared library reused by the Haskell server via
  `foreign import ccall`,
- **native CLI** — the self-contained `rdf-shapes` binary.

It replaces JVM/Apache-Jena CLI dependencies for querying and validating RDF
transaction graphs.

## Engines

- [Oxigraph](https://github.com/oxigraph/oxigraph) — SPARQL 1.1 query
- [rudof](https://github.com/rudof-project/rudof) — SHACL Core validation

## Status

The engine and the browser playground are **shipped**. `rdf-shapes-core`
implements full **SPARQL 1.1** (Oxigraph) and **SHACL Core** (rudof) over
in-memory Turtle, surfaced through three reuse contracts — the browser wasm
bundle, the native FFI lib, and the native CLI — all built reproducibly by Nix.

- **Docs:** <https://lambdasistemi.github.io/rdf-shapes-wasm/>
- **Live playground:** <https://lambdasistemi.github.io/rdf-shapes-wasm/app/>

Built and tested entirely with Nix — `nix flake check` / `just ci`. Rust
toolchain via [crane](https://github.com/ipetkov/crane) + rust-overlay. W3C
conformance and the Jena differential harness are wired into the gate; see
[`docs/conformance.md`](docs/conformance.md).

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
| `just conformance` | Run the Apache Jena differential and curated W3C conformance harness. |
| `nix build .#cli` | Native CLI; run `./result/bin/rdf-shapes --help`. |
| `nix build .#wasm-pkg` | Reproducible, npm-shaped wasm bundle (pinned `wasm-bindgen-cli` + `wasm-opt -Oz`). |
| `nix build .#ffi-lib` | Native C-ABI shared library: `lib/librdf_shapes_ffi.{so,dylib}` + `include/rdf_shapes.h` (cbindgen-generated). |
| `nix build .#release-artifacts` | CLI tarball + npm `.tgz` + bare `.wasm` + native FFI lib tarball + `SHA256SUMS`. |

The `.wasm` is reproducible: two clean builds of `.#wasm-pkg` yield a
byte-identical artifact. The `wasm-bindgen` library version is locked to the
pinned `wasm-bindgen-cli` (see `Cargo.toml` and `flake.nix`).

## Downstream reuse

The two reuse contracts are flake outputs, so a downstream project consumes
them by adding this repo as a flake input and referencing the package for its
system. Both are Nix-built and verified consumable.

### Browser wasm (`wasm-pkg`)

The npm-shaped bundle: `rdf_shapes_wasm.js` (the `wasm-bindgen` JS shim) and
`rdf_shapes_wasm_bg.wasm`, plus TypeScript types and a `package.json`.

```nix
{
  inputs.rdf-shapes-wasm.url = "github:lambdasistemi/rdf-shapes-wasm";

  outputs = { self, nixpkgs, rdf-shapes-wasm, ... }:
    let
      system = "x86_64-linux";
      wasm = rdf-shapes-wasm.packages.${system}.wasm-pkg;
    in {
      # `${wasm}` contains rdf_shapes_wasm.js + rdf_shapes_wasm_bg.wasm;
      # copy them into your web bundle at build time.
    };
}
```

```js
import init, { start, query, validate } from "./rdf_shapes_wasm.js";
await init();
start();
const result = query(graphTtl, sparql);
const report = validate(dataTtl, shapesTtl);
```

### Native FFI lib (`ffi-lib`)

The C-ABI shared library — `lib/librdf_shapes_ffi.{so,dylib}` plus the
cbindgen-generated `include/rdf_shapes.h` — consumed from Haskell via
`foreign import ccall` (verified on GHC 9.12.3). This is the **server** reuse
path: the engine runs natively in-process, not on a wasm host.

```nix
{
  inputs.rdf-shapes-wasm.url = "github:lambdasistemi/rdf-shapes-wasm";

  outputs = { self, nixpkgs, rdf-shapes-wasm, ... }:
    let
      system = "x86_64-linux";
      ffi = rdf-shapes-wasm.packages.${system}.ffi-lib;
    in {
      # `${ffi}/lib/librdf_shapes_ffi.so` links into the Haskell server;
      # `${ffi}/include/rdf_shapes.h` is the matching C header.
    };
}
```

```haskell
foreign import ccall "rdf_shapes_query"
  rdf_shapes_query :: CString -> CString -> IO CString
```

Each FFI function returns a JSON envelope — `{"ok": <result>}` on success,
`{"error": "<message>"}` on failure — and the caller frees the returned string
with `rdf_shapes_string_free`. See
[`crates/rdf-shapes-ffi/smoke`](crates/rdf-shapes-ffi/smoke) for the GHC 9.12.3
`ccall` proof.

## Releasing

Releases are **tag-driven** and reproducible — no release-please (the repo is a
virtual cargo workspace with an inherited version, which release-please can't
manage cleanly). To cut `vX.Y.Z`:

```bash
just release X.Y.Z          # bumps [workspace.package].version + Cargo.lock,
                            # stamps a CHANGELOG section, commits chore(release)
# edit the CHANGELOG entry, open + merge a PR for that commit, then:
git tag vX.Y.Z && git push origin vX.Y.Z
```

Pushing the `v*` tag runs `.github/workflows/release.yml`, which builds
`.#release-artifacts` with Nix and publishes the GitHub release (CLI tarball,
npm `.tgz`, bare `.wasm`, native FFI lib tarball, `SHA256SUMS`). The npm publish
step is a no-op until an `NPM_TOKEN` secret is set.

## License

[Apache-2.0](./LICENSE)
