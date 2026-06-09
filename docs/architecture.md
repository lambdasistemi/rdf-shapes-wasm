# Architecture

How the workspace is laid out, how Nix builds it reproducibly, and how
correctness is established.

## Pure core, thin shells

The workspace follows a strict **pure-core / thin-shells** discipline.
All business logic lives in one portable crate; the other two crates are
thin marshalling layers over it.

| Crate | Role | Constraints |
|---|---|---|
| `rdf-shapes-core` | All business logic. The query and validate engines live here. | Builds **native and** `wasm32-unknown-unknown`. No `wasm-bindgen`, no I/O, no threads, no filesystem, no sockets. |
| `rdf-shapes-wasm` | `#[wasm_bindgen]` shims. Produces the `.wasm`. | `crate-type = ["cdylib", "rlib"]`. Thin marshalling only — every export delegates to core. |
| `rdf-shapes-cli` | The native `rdf-shapes` binary. | A thin `clap` front end over core. No logic of its own. |

The `cdylib` half of the wasm crate produces the `.wasm`; the `rlib`
half keeps it unit-testable natively. Because core has no host
dependencies, the *exact same* logic runs in the browser, on a server
wasm host, and in the native CLI. Keeping host concerns out of core is
what makes "build once, run everywhere" true rather than aspirational.

```mermaid
flowchart TD
    core["rdf-shapes-core<br/>(portable logic:<br/>SPARQL + SHACL)"]
    wasmcrate["rdf-shapes-wasm<br/>(#[wasm_bindgen] cdylib)"]
    cli["rdf-shapes-cli<br/>(native clap binary)"]

    core --> wasmcrate
    core --> cli

    wasmcrate --> artifact[".wasm + JS shim"]

    artifact --> browser["Browser<br/>(client-side dashboard)"]
    artifact --> server["Server<br/>(wasm host)"]
    cli --> native["Native CLI<br/>(CI, scripts)"]
```

## The Nix build

Everything is built by Nix, so the build is identical locally and in CI.

- **[crane](https://github.com/ipetkov/crane)** drives the Cargo build.
  Its `buildDepsOnly` produces a content-addressed, dependency-only
  artifact that the CLI build, the library build, and every check
  (clippy, fmt, nextest, deny, doc) reuse — so the store is warmed once
  and downstream work is near-instant. Two dependency-only artifact sets
  are produced: one for the native host triple and one cross-compiled to
  `wasm32`.
- **[rust-overlay](https://github.com/oxalica/rust-overlay)** provides
  the toolchain, read from a single `rust-toolchain.toml` (an explicit
  stable channel, bumped deliberately).
- `flake.nix` stays thin and imports `nix/*.nix`. No
  `buildRustPackage`, no `naersk`.

## Reproducible wasm

The `.wasm` is a project deliverable, so it must be byte-reproducible
and dependency-free.

1. crane compiles the `cdylib` for `wasm32-unknown-unknown`.
2. A **pinned `wasm-bindgen-cli`** runs over it
   (`--target web --omit-default-module-path`). Its version **must equal**
   the `wasm-bindgen` library version in `Cargo.lock`, down to the patch
   — they are bumped together in one commit. A mismatch produces cryptic
   *runtime* errors, so the pin is non-negotiable.
3. **`wasm-opt -Oz`** (binaryen) size-optimizes the module, with
   `--enable-bulk-memory --enable-mutable-globals` because the stable
   toolchain emits `memory.copy` / `memory.fill`.

No `wasm-pack` (it fetches its own toolchain and is not reproducible),
and no network access in the sandbox.

**Double-build verification:** two clean builds of `.#wasm-pkg` yield a
byte-identical artifact. Compare the SHA-256:

```bash
nix build .#wasm-pkg && sha256sum result/rdf_shapes_wasm_bg.wasm
nix build .#wasm-pkg --rebuild && sha256sum result/rdf_shapes_wasm_bg.wasm
```

`SHA256SUMS` ships with each release so consumers can rebuild and verify.

## Trust model: conformance and differential parity

The project replaces a trusted engine (Apache Jena), so correctness is
established by **evidence, not assertion**:

- **Conformance** — the W3C SPARQL 1.1 query test suite and the W3C
  SHACL test suite. Any known-unsupported feature is an explicit,
  documented skip — never a silent one.
- **Differential / Jena oracle** — a harness runs the same graph plus
  query or shape through both Apache Jena (`arq` / `shacl`, the
  reference oracle) and this engine, and asserts identical results over
  a seed corpus. The engine must achieve parity on the seed corpus
  before it may replace Jena in any consumer.

Jena is the oracle **only** — it is never a runtime dependency of the
shipped artifact. Measured parity against it is the thing that licenses
deleting it. There is no formal-proof gate here: packaging an existing
engine has no novel state-machine invariant to prove, so conformance and
differential parity *are* the arbiter of correctness.

## The single gate

`nix flake check` is the one source of truth, equal to `just ci` and to
CI. It runs, as real sandboxed derivations:

- `clippy` with `-D warnings`,
- `rustfmt` in check mode,
- `nextest` (the unit test suite),
- `cargo-deny` (licenses, advisories, bans, sources),
- `cargo doc` with `-D warnings`.

A change is not "done" until that command is green; "it compiles" is not
sufficient. See [Usage](usage.md) for the day-to-day commands.
