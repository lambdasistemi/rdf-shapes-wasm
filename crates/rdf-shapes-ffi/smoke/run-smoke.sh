#!/usr/bin/env bash
# Native Haskell ccall smoke (issue #19): link the C-ABI cdylib from
# `nix build .#ffi-lib` and drive the engine through
# `foreign import ccall` on GHC 9.12.3 — the live-boundary proof that
# the Haskell backend (replacing Jena) can use the engine natively.
#
# Run from anywhere; it builds `.#ffi-lib` and fetches GHC 9.12.3 from
# nixpkgs itself, so the only prerequisite is Nix:
#
#   crates/rdf-shapes-ffi/smoke/run-smoke.sh
#
# Exits non-zero on any link error, runtime error, or unmet
# expectation, so it fails loudly.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"

# 1. Build the C-ABI shared library + header.
ffi="$(nix build "$root#ffi-lib" --no-link --print-out-paths)"
echo "ffi-lib: $ffi"
echo "header : $ffi/include/rdf_shapes.h"

# 2. Compile the ccall smoke on GHC 9.12.3, linking the cdylib, and run
#    it. GHC and the linker come from nixpkgs; the rpath points the
#    loader at the cdylib's store path.
work="$(mktemp -d)"
cp "$here/Main.hs" "$work/Main.hs"

nix shell nixpkgs#haskell.compiler.ghc9123 -c bash -c "
  set -euo pipefail
  cd '$work'
  ghc --version
  ghc -O0 Main.hs \
    -L'$ffi/lib' -lrdf_shapes_ffi \
    -optl-Wl,-rpath,'$ffi/lib' \
    -o smoke
  LD_LIBRARY_PATH='$ffi/lib' ./smoke
"
