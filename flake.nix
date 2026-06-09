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
  };

  outputs =
    { nixpkgs
    , flake-utils
    , crane
    , rust-overlay
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = import ./nix/toolchain.nix { inherit pkgs; };

        craneEnv = import ./nix/crane.nix {
          inherit pkgs crane rustToolchain;
          src = ./.;
        };

        # The pinned wasm-bindgen-cli. Its version MUST equal the
        # `wasm-bindgen` library version in Cargo.toml/Cargo.lock.
        # SPIKE (#3): bumped to 0.2.108 to match the wasm-bindgen lib
        # version forced by oxigraph 0.5.8's js-sys 0.3.85.
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
      in
      {
        packages = {
          default = packages.cli;
          inherit (packages) cli lib wasm-pkg;
          inherit api-docs;
          release-artifacts = import ./nix/release.nix {
            inherit pkgs packages;
          };
        };

        inherit checks apps;

        devShells.default = craneEnv.craneLib.devShell {
          packages = [
            pkgs.just
            pkgs.cargo-deny
            pkgs.binaryen
            wasmBindgenCli
          ];
        };
      }
    );
}
