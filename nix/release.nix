# Assembles the tag-driven release bundle: a native CLI tarball, the
# npm-shaped wasm package as a .tgz, the bare optimized .wasm, the
# native FFI library tarball (the server reuse contract), and a
# SHA256SUMS manifest over all of them. Everything is Nix-built from
# the committed Cargo.lock with no network access, so the bundle is
# reproducible byte for byte.
{ pkgs, packages }:
let
  inherit (packages) cli wasm-pkg ffi-lib;
  version = "0.1.0";
  # System triple is part of the FFI tarball name: the C-ABI library is
  # host-specific (a `.so` on Linux, a `.dylib` on Darwin), unlike the
  # portable wasm and the statically-linkable CLI.
  inherit (pkgs.stdenv.hostPlatform) system;
in
pkgs.runCommand "rdf-shapes-release-artifacts"
{
  nativeBuildInputs = [
    pkgs.gnutar
    pkgs.gzip
    pkgs.coreutils
  ];
}
  ''
    set -euo pipefail
    mkdir -p "$out"

    # Native CLI tarball.
    tar -czf "$out/rdf-shapes-${version}-cli.tar.gz" \
      -C ${cli}/bin rdf-shapes

    # npm-shaped wasm package tarball (npm expects a top-level
    # `package/` directory inside the .tgz).
    pkgdir="$(mktemp -d)/package"
    mkdir -p "$pkgdir"
    cp -r ${wasm-pkg}/* "$pkgdir/"
    tar -czf "$out/rdf-shapes-wasm-${version}.tgz" \
      -C "$(dirname "$pkgdir")" package

    # Bare optimized wasm binary.
    cp ${wasm-pkg}/rdf_shapes_wasm_bg.wasm \
      "$out/rdf_shapes_wasm_bg.wasm"

    # Native FFI library tarball: the C-ABI shared library plus its
    # cbindgen-generated header, the server/Haskell reuse contract. The
    # tarball preserves the `lib/` + `include/` layout the consumer
    # links and includes against.
    ffidir="$(mktemp -d)/rdf-shapes-ffi"
    mkdir -p "$ffidir"
    cp -r ${ffi-lib}/lib "$ffidir/lib"
    cp -r ${ffi-lib}/include "$ffidir/include"
    tar -czf "$out/rdf-shapes-ffi-${system}.tar.gz" \
      -C "$(dirname "$ffidir")" rdf-shapes-ffi

    # Checksums over every shipped artifact.
    ( cd "$out" && sha256sum -- * > SHA256SUMS )
  ''
