# Repository Agent Guide

## What this repo is

`rdf-shapes-wasm` is a portable **SPARQL 1.1** query + **SHACL Core**
validation engine. One Rust core (`rdf-shapes-core`) holds all the logic
— SPARQL via [Oxigraph](https://github.com/oxigraph/oxigraph), SHACL Core
via [rudof](https://github.com/rudof-project/rudof), over in-memory
Turtle — and is wrapped by thin shells that ship it to three targets: a
browser `wasm-bindgen` bundle, a native C-ABI FFI library, and a native
CLI (`rdf-shapes`). It replaces JVM/Apache-Jena CLI tooling; Jena
survives only as a differential oracle in the conformance harness.
Everything is built and tested with Nix.

## How to work here

Everything goes through Nix, so the gate is identical locally and in CI.

- **The single gate:** `nix flake check` — clippy (`-D warnings`),
  rustfmt, nextest, cargo-deny, rustdoc (`-D warnings`), the conformance
  harness, and the playground check.
- **CI gate via apps:** `just ci` (`nix run .#ci`) builds every package
  plus the Rust checks.
- **Build the CLI:** `nix build .#cli` → `./result/bin/rdf-shapes`.
- **Other packages:** `nix build .#wasm-pkg` (browser bundle),
  `nix build .#ffi-lib` (C-ABI lib + header), `nix build .#playground`
  (browser app), `nix build .#release-artifacts`.
- **Focused recipes:** `just build`, `just test`, `just clippy`,
  `just fmt-check`, `just wasm`, `just ffi`, `just conformance`,
  `just deny`. Run `just --list` for the full set; `just fmt` formats
  in place.
- **Format:** Rust via `cargo fmt` (run inside `nix develop`); the
  PureScript playground pins `purs-tidy 0.10.0`.

The workspace forbids `unsafe_code` everywhere except `rdf-shapes-ffi`,
which scopes it to the C-ABI pointer marshalling. The `.wasm` is
byte-reproducible and the `wasm-bindgen` library version is locked to the
pinned `wasm-bindgen-cli` (`Cargo.toml` + `flake.nix`) — bump them
together.

## Skills

Activatable procedures live under `skills/`. Load the one whose
description matches your task.

- `skills/rdf-shapes-wasm-guide/` — repository map, build/test/run
  commands, where the SPARQL/SHACL logic lives, how to use the CLI / wasm
  / FFI surfaces, and where the answers to common questions live.

## Further reading

- Human entry point: [README.md](README.md).
- Documentation site: <https://lambdasistemi.github.io/rdf-shapes-wasm/>
  (concepts, architecture, conformance, usage, the live playground, and
  the rustdoc API reference).
