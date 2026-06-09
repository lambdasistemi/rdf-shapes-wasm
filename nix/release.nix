# Assembles the tag-driven release bundle: a native CLI tarball, the
# npm-shaped wasm package as a .tgz, the bare optimized .wasm, and a
# SHA256SUMS manifest over all of them. Everything is Nix-built from
# the committed Cargo.lock with no network access, so the bundle is
# reproducible byte for byte.
{ pkgs, packages }:
let
  inherit (packages) cli wasm-pkg;
  version = "0.1.0";
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

    # Checksums over every shipped artifact.
    ( cd "$out" && sha256sum -- * > SHA256SUMS )
  ''
