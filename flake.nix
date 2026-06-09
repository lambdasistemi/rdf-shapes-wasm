{
  description = "rdf-shapes-wasm: portable RDF shapes core, wasm shim, and CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # PureScript toolchain for the browser playground (`app/`). The
    # paolino fork drops the broken nodePackages dependency; pins
    # purs / spago-unstable / purs-tidy-0_10_0 deterministically.
    purescript-overlay = {
      url = "github:paolino/purescript-overlay/fix/remove-nodePackages";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # Reproducible `spago bundle` inside a sandboxed Nix derivation.
    mkSpagoDerivation = {
      url = "github:jeslie0/mkSpagoDerivation";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs
    , flake-utils
    , crane
    , rust-overlay
    , purescript-overlay
    , mkSpagoDerivation
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
            purescript-overlay.overlays.default
            mkSpagoDerivation.overlays.default
          ];
        };

        rustToolchain = import ./nix/toolchain.nix { inherit pkgs; };

        craneEnv = import ./nix/crane.nix {
          inherit pkgs crane rustToolchain;
          src = ./.;
        };

        # The pinned wasm-bindgen-cli. Its version MUST equal the
        # `wasm-bindgen` library version in Cargo.toml/Cargo.lock.
        # Bumped to 0.2.108 in lockstep with the wasm-bindgen lib
        # (forced by oxigraph 0.5.8's js-sys 0.3.85; Constitution II).
        wasmBindgenCli = pkgs.wasm-bindgen-cli_0_2_108;

        packages = import ./nix/packages.nix {
          inherit pkgs craneEnv wasmBindgenCli;
        };

        checks = import ./nix/checks.nix { inherit pkgs craneEnv; };

        apps = import ./nix/apps.nix { inherit pkgs; };

        # Reproducible rustdoc HTML, co-hosted under the docs site at
        # `/api/`. Publishes even with doc warnings; the `doc` check
        # enforces clean docs separately.
        api-docs = import ./nix/api-docs.nix { inherit craneEnv; };

        # The PureScript/Halogen browser playground (`app/`), built by
        # mkSpagoDerivation and fed the reproducible `wasm-pkg` output.
        playground = import ./nix/playground.nix {
          inherit pkgs;
          wasmPkg = packages.wasm-pkg;
        };
      in
      {
        packages = {
          default = packages.cli;
          inherit (packages) cli lib wasm-pkg ffi-lib;
          inherit api-docs;
          playground = playground.bundle;
          release-artifacts = import ./nix/release.nix {
            inherit pkgs packages;
          };
        };

        checks = checks // {
          playground = playground.check;
        };

        inherit apps;

        devShells.default = craneEnv.craneLib.devShell {
          packages = [
            pkgs.just
            pkgs.cargo-deny
            pkgs.binaryen
            wasmBindgenCli
            # PureScript playground toolchain.
            pkgs.nodejs_22
            pkgs.purs
            pkgs.spago-unstable
            pkgs.purs-tidy-bin.purs-tidy-0_10_0
            pkgs.purescript-language-server
            pkgs.esbuild
          ];
        };
      }
    );
}
