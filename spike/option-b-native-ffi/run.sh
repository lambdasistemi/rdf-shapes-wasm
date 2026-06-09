#!/usr/bin/env bash
# SPIKE (#5) Option B smoke runner: native Rust cdylib via Haskell FFI.
#
# 1. builds the native cdylib (librdf_shapes_server_host.so) with cargo
# 2. compiles Smoke.hs with GHC, linking against that cdylib
# 3. runs it, with the cdylib on the loader path
#
# Everything runs inside the repo's `nix develop` shell, which already
# provides cargo. GHC is pulled in ad hoc via `nix shell nixpkgs#ghc`
# (this spike does NOT add GHC to the Rust flake's dev shell).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"

echo "==> building native cdylib"
( cd "$repo" && nix develop --quiet -c cargo build -p rdf-shapes-server-host --release )

libdir="$repo/target/release"
test -f "$libdir/librdf_shapes_server_host.so" \
    || test -f "$libdir/librdf_shapes_server_host.dylib"

echo "==> compiling + running the Haskell FFI smoke"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT

nix shell nixpkgs#ghc --quiet -c \
    ghc -O0 -outputdir "$out" -o "$out/smoke" "$here/Smoke.hs" \
        -L"$libdir" -lrdf_shapes_server_host -optl-Wl,-rpath,"$libdir"

LD_LIBRARY_PATH="$libdir${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
    DYLD_LIBRARY_PATH="$libdir${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" \
    "$out/smoke"
