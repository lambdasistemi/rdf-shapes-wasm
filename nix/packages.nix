# Deliverable packages: the native CLI, the native core library, the
# raw wasm cdylib, and the finalized npm-shaped wasm bundle.
{ pkgs
, craneEnv
, wasmBindgenCli
}:
let
  inherit (craneEnv)
    craneLib
    commonArgs
    cargoArtifacts
    wasmArgs
    cargoArtifactsWasm
    ;

  # Native CLI binary (the `rdf-shapes` executable).
  cli = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      pname = "rdf-shapes-cli";
      cargoExtraArgs = "-p rdf-shapes-cli";
    }
  );

  # Native build of the portable core library.
  lib = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      pname = "rdf-shapes-core";
      cargoExtraArgs = "-p rdf-shapes-core";
    }
  );

  # The raw cdylib compiled to wasm32, before wasm-bindgen processing.
  wasmCrate = craneLib.buildPackage (
    wasmArgs
    // {
      cargoArtifacts = cargoArtifactsWasm;
      pname = "rdf-shapes-wasm";
    }
  );

  # Native C-ABI shared library over the engine — the server/Haskell
  # reuse contract, the native sibling of `wasm-pkg`. crane builds the
  # `cdylib` to `$out/lib/librdf_shapes_ffi.{so,dylib}`; a post-build
  # step runs the pinned `cbindgen` over the crate to emit the matching
  # C header into `$out/include/rdf_shapes.h`. The header is generated
  # (not committed) so it can never drift from the exported symbols.
  #
  # cbindgen shells out to `cargo metadata`, so cargo from the pinned
  # toolchain is on PATH inside the crane builder; `parse_deps = false`
  # in cbindgen.toml keeps it from descending into dependencies, so the
  # generation is offline and deterministic.
  ffi-lib = craneLib.buildPackage (
    commonArgs
    // {
      inherit cargoArtifacts;
      pname = "rdf-shapes-ffi";
      cargoExtraArgs = "-p rdf-shapes-ffi";

      nativeBuildInputs = [ pkgs.rust-cbindgen ];

      postInstall = ''
        mkdir -p "$out/include"
        cbindgen \
          --config crates/rdf-shapes-ffi/cbindgen.toml \
          --crate rdf-shapes-ffi \
          --lang c \
          --output "$out/include/rdf_shapes.h" \
          crates/rdf-shapes-ffi
      '';
    }
  );

  version = commonArgs.version;
  npmName = "@lambdasistemi/rdf-shapes-wasm";

  # Finalize the cdylib into a web-target bundle with a pinned
  # wasm-bindgen-cli (version == the wasm-bindgen lib in Cargo.lock),
  # then size-optimize with wasm-opt. No wasm-pack, no network.
  wasm-pkg = pkgs.stdenvNoCC.mkDerivation {
    pname = "rdf-shapes-wasm-pkg";
    inherit version;

    src = wasmCrate;

    nativeBuildInputs = [
      wasmBindgenCli
      pkgs.binaryen
    ];

    # Deterministic: the same cdylib input yields the same bytes on
    # every build (verified by the double-build SHA-256 check).
    buildPhase = ''
      runHook preBuild

      mkdir -p pkg
      wasm-bindgen \
        --target web \
        --omit-default-module-path \
        --out-dir pkg \
        --out-name rdf_shapes_wasm \
        lib/rdf_shapes_wasm.wasm

      # The pinned stable toolchain emits bulk-memory instructions
      # (memory.copy / memory.fill); enable them so wasm-opt validates
      # and optimizes the module instead of rejecting it.
      wasm-opt -Oz \
        --enable-bulk-memory \
        --enable-mutable-globals \
        -o pkg/rdf_shapes_wasm_bg.wasm \
        pkg/rdf_shapes_wasm_bg.wasm

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall

      mkdir -p "$out"
      cp pkg/* "$out/"

      cat > "$out/package.json" <<EOF
      {
        "name": "${npmName}",
        "version": "${version}",
        "description": "Reproducible wasm bundle for rdf-shapes-wasm.",
        "license": "Apache-2.0",
        "type": "module",
        "main": "rdf_shapes_wasm.js",
        "types": "rdf_shapes_wasm.d.ts",
        "files": [
          "rdf_shapes_wasm_bg.wasm",
          "rdf_shapes_wasm.js",
          "rdf_shapes_wasm.d.ts"
        ],
        "repository": {
          "type": "git",
          "url": "https://github.com/lambdasistemi/rdf-shapes-wasm"
        }
      }
      EOF

      runHook postInstall
    '';
  };
in
{
  inherit cli lib wasmCrate wasm-pkg ffi-lib;
}
