# Usage

How to build and run the project today. Everything runs through Nix, so
the commands are identical locally and in CI.

!!! note "Scaffold surface"
    The public surface today is intentionally trivial: `version` and
    `ping`. They prove the toolchain end to end — a value flows from a
    caller, through the portable core, and back out, across both the
    native CLI and the wasm shim. The real query and validate APIs land
    with engine work tracked in
    [#6](https://github.com/lambdasistemi/rdf-shapes-wasm/issues/6).

## Building

Build the deliverables straight from the flake:

```bash
# Native CLI (the `rdf-shapes` executable)
nix build .#cli

# Portable core library (native build)
nix build .#lib

# Reproducible, npm-shaped wasm bundle
nix build .#wasm-pkg

# CLI tarball + npm .tgz + bare .wasm + SHA256SUMS
nix build .#release-artifacts

# This documentation's Rust API reference (rustdoc HTML)
nix build .#api-docs
```

## The CLI today

After `nix build .#cli`, the binary is at `result/bin/rdf-shapes`:

```console
$ ./result/bin/rdf-shapes --help
Trivial scaffold CLI over rdf-shapes-core

Usage: rdf-shapes <COMMAND>

Commands:
  version  Print the crate version reported by the core
  ping     Echo a value back through the core's `ping`
  help     Print this message or the help of the given subcommand(s)

$ ./result/bin/rdf-shapes version
0.1.0

$ ./result/bin/rdf-shapes ping hello
pong: hello
```

## The wasm bundle today

`nix build .#wasm-pkg` produces a web-target bundle (`.wasm`, the
`wasm-bindgen` JS shim, TypeScript types, and a `package.json`). The
exported functions mirror the CLI surface:

```js
import init, { start, version, ping } from "./rdf_shapes_wasm.js";

await init();
start();                 // installs the panic hook
version();               // "0.1.0"
ping("hello");           // "pong: hello"
```

## The gate

| Command | What it does |
|---|---|
| `nix flake check` | The single gate: clippy (`-D warnings`), rustfmt, nextest, cargo-deny, rustdoc (`-D warnings`). |
| `just ci` | The same gate via `nix run .#ci`. |
| `just build` | Native CLI + core library. |
| `just test` | Unit tests (cargo-nextest). |
| `just clippy` | Clippy with warnings denied. |
| `just fmt-check` | rustfmt in check mode. |
| `just wasm` | The reproducible wasm bundle. |

Run `just --list` for the full set. A change is not "done" until
`nix flake check` is green.

## Coming next

The engine spikes and the conformance / Jena-differential harness arrive
with later work — see the
[issues](https://github.com/lambdasistemi/rdf-shapes-wasm/issues) and
the epic [#1](https://github.com/lambdasistemi/rdf-shapes-wasm/issues/1).
Once the real APIs land, this page grows query and validate examples for
both the CLI and the browser.
