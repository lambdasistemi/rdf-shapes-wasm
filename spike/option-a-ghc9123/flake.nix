{
  # SPIKE (#5) Option A — THE crux probe: can `wasmtime-hs` be built
  # under GHC 9.12.3 (`ghc9123`), the org default that amaru-treasury-tx
  # targets?
  #
  # This flake deliberately mirrors amaru-treasury-tx's haskell.nix setup
  # as closely as a standalone probe can: the SAME haskell.nix pin, the
  # SAME `compiler-nix-name = "ghc9123"`, and an `index-state` from the
  # same era. wasmtime-hs is pulled in as a `source-repository-package`
  # (it is NOT on Hackage), and the wasmtime C-API library is patched into
  # `pkgs.wasmtime` exactly the way wasmtime-hs's own flake does it (a
  # `cmake -S crates/c-api` overlay), because plain `pkgs.wasmtime` does
  # NOT ship the C-API headers/lib that `extra-libraries: wasmtime` needs.
  #
  # The point of the probe is one boolean: does `wasmtime-hs` COMPILE
  # under ghc9123 with a contemporary index-state, or do its dependency
  # bounds (`primitive < 0.9`, `transformers < 0.7`, `vector < 0.14`)
  # collide with the GHC 9.12.3 boot libraries / the Hackage snapshot?
  description = "spike#5 probe: wasmtime-hs under ghc9123";

  nixConfig = {
    extra-substituters = [ "https://cache.iog.io" ];
    extra-trusted-public-keys =
      [ "hydra.iohk.io:f/Ea+s+dFdN+3Y/G+FDgSq+a5NEWhJGzdjvKNGv0/EQ=" ];
  };

  inputs = {
    # SAME haskell.nix + hackage.nix pins as amaru-treasury-tx/flake.nix.
    haskellNix = {
      url =
        "github:input-output-hk/haskell.nix/8b447d7f57d62fab9249f79bb916bc891e29b9d0";
      inputs.hackage.follows = "hackageNix";
    };
    hackageNix = {
      url =
        "github:input-output-hk/hackage.nix/b6b4aa4bd699f743238da45c7f43da5a26a822f7";
      flake = false;
    };
    nixpkgs.follows = "haskellNix/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, haskellNix, flake-utils, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-darwin" ]
    (system:
      let
        overlays = [
          haskellNix.overlay
          # wasmtime-hs's own C-API overlay, copied verbatim in spirit:
          # build + install the wasmtime C-API so `-lwasmtime` resolves.
          (self: super: {
            wasmtime = super.wasmtime.overrideAttrs (old: {
              postInstall = ''
                ${old.postInstall or ""}
                cmake -S crates/c-api -B target/c-api \
                  --install-prefix "$(pwd)/artifacts"
                cmake --build target/c-api
                cmake --install target/c-api
                install -m0644 \
                  $(pwd)/artifacts/include/wasmtime/conf.h \
                  $dev/include/wasmtime
              '';
            });
          })
        ];
        pkgs = import nixpkgs { inherit system overlays; inherit (haskellNix) config; };

        project = pkgs.haskell-nix.cabalProject' {
          name = "wasmtime-hs-ghc9123-probe";
          src = ./.;
          # THE pivotal line: force the org default GHC.
          compiler-nix-name = "ghc9123";
          # Contemporary Hackage snapshot, same era as amaru-treasury-tx
          # (whose cabal.project pins 2026-02-17). This is what decides
          # whether `primitive`/`vector`/`transformers` resolve within
          # wasmtime-hs's upper bounds.
          index-state = "2026-02-17T10:15:41Z";
          modules = [{
            # Point the wasmtime-hs library's `extra-libraries: wasmtime`
            # at the C-API-patched wasmtime.
            packages.wasmtime-hs.components.library.libs =
              pkgs.lib.mkForce [ pkgs.wasmtime ];
          }];
        };
      in {
        packages.wasmtime-hs = project.hsPkgs.wasmtime-hs.components.library;
        packages.default = self.packages.${system}.wasmtime-hs;
        # Expose the plan so we can read the resolved versions even if the
        # build fails to compile.
        legacyPackages = project;
      });
}
