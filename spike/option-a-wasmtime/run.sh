#!/usr/bin/env bash
# SPIKE (#5) Option A smoke runner: load the plain wasm module from
# Haskell via wasmtime-hs and read a string out of linear memory.
#
# NOTE: this path enters wasmtime-hs's OWN nix dev shell, which pins
# GHC 9.6.6 (its nixpkgs default) + wasmtime v29 — NOT the org default.
# It is kept only to demonstrate the linear-memory marshalling on the
# compiler/engine combo wasmtime-hs actually supports. For the binding
# decision use spike/option-a-ghc9123/ (the haskell.nix probe that pins
# compiler-nix-name = "ghc9123"); that is where Option A is shown to be
# blocked on the org default. See spike/VERDICT.md.
#
# 1. builds the WASI-loadable wasm module (NOT the wasm-bindgen bundle)
# 2. builds wasmtime-hs (+ its patched wasmtime C-API lib) from upstream
# 3. compiles Smoke.hs in a GHC environment that has wasmtime-hs and
#    libwasmtime, then runs it against the wasm module
#
# wasmtime-hs is NOT on Hackage and needs a patched `pkgs.wasmtime`
# (its flake adds the C-API via a cmake overlay). The simplest faithful
# way to get a GHC that can see the package is to enter wasmtime-hs's own
# `nix develop` shell (which provides GHC + libwasmtime + the package's
# dependency closure) and build the smoke there with the package in
# scope. Adjust WASMTIME_HS to a local clone of dfinity/wasmtime-hs.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
WASMTIME_HS="${WASMTIME_HS:-/tmp/wasmtime-hs-probe}"

echo "==> building the WASI-loadable wasm module"
( cd "$repo" \
    && nix develop --quiet -c \
        cargo build -p rdf-shapes-server-host \
        --target wasm32-unknown-unknown --release )

wasm="$repo/target/wasm32-unknown-unknown/release/rdf_shapes_server_host.wasm"
test -f "$wasm"

if [ ! -d "$WASMTIME_HS" ]; then
    echo "ERROR: wasmtime-hs clone not found at $WASMTIME_HS" >&2
    echo "  git clone https://github.com/dfinity/wasmtime-hs $WASMTIME_HS" >&2
    exit 1
fi

echo "==> building + running the Haskell wasmtime-hs smoke"
# Enter wasmtime-hs's dev shell so GHC, libwasmtime, and the package's
# dependency closure are all present. We build the library, write a GHC
# environment file that exposes the just-built inplace `wasmtime-hs`
# package, then compile + run the smoke against it. (`cabal build` alone
# does not register the inplace lib in a standalone package.conf.d, so a
# v1-style `-package-db` lookup comes back empty — the env file is the
# robust path.)
( cd "$WASMTIME_HS" \
    && nix develop --quiet -c bash -euo pipefail -c "
        set -x
        cabal build wasmtime-hs
        cabal install --lib wasmtime-hs \
            --package-env '$here/smoke.env' --force-reinstalls
        ghc -O0 -package-env '$here/smoke.env' \
            -o /tmp/option-a-smoke '$here/Smoke.hs'
        /tmp/option-a-smoke '$wasm'
    " )
