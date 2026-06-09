# Builds the crane library bound to the pinned toolchain and the
# shared argument records used by both packages and checks.
#
# Two dependency-only artifact sets are produced: one for the native
# host triple (used by the CLI, the core lib, clippy, fmt, nextest,
# deny, and doc) and one cross-compiled to wasm32 (used by the wasm
# cdylib package). Splitting them keeps incremental rebuilds tight.
{ pkgs
, crane
, rustToolchain
, src
}:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  # crane's cleanCargoSource keeps only Rust/Cargo files; the core
  # crate `include_str!`s Turtle testdata, so union the cargo sources
  # with `.ttl` files (test fixtures) to keep them in the build closure.
  src' = pkgs.lib.cleanSourceWith {
    src = craneLib.path src;
    filter =
      path: type:
      (craneLib.filterCargoSources path type)
      || (pkgs.lib.hasSuffix ".ttl" path);
  };

  commonArgs = {
    src = src';
    strictDeps = true;
    pname = "rdf-shapes-wasm";
    version = "0.1.0";

    buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.libiconv
    ];
  };

  # Native host dependency artifacts, shared across the native
  # packages and every check.
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  # Cross-compilation arguments for the wasm cdylib. The wasm crate is
  # the only member built for this target, and it has no tests to run
  # under wasm, so checks are disabled here.
  #
  # getrandom_backend="wasm_js" selects getrandom 0.3's JS backend on
  # wasm. oxigraph's `js` feature is happy without it, but rudof's
  # transitive getrandom REQUIRES the cfg; setting it here is the
  # superset that satisfies both engines in the combined crate. It is
  # scoped to the wasm artifacts only — the native build never sees it.
  # Mirrored in .cargo/config.toml so a bare `cargo check
  # --target wasm32-unknown-unknown` resolves the same way.
  wasmArgs = commonArgs // {
    CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
    cargoExtraArgs = "-p rdf-shapes-wasm";
    CARGO_BUILD_RUSTFLAGS = ''--cfg getrandom_backend="wasm_js"'';
    doCheck = false;
  };

  cargoArtifactsWasm = craneLib.buildDepsOnly wasmArgs;
in
{
  inherit
    craneLib
    commonArgs
    cargoArtifacts
    wasmArgs
    cargoArtifactsWasm
    ;
}
