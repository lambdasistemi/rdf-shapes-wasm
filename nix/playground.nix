# The PureScript/Halogen browser playground (`app/`).
#
# Built by mkSpagoDerivation from the committed spago.lock, consuming
# the reproducible `wasm-pkg` derivation (no npm install, no network):
#   1. vendor the wasm-pkg ESM + _bg.wasm into vendor/
#   2. esbuild bootstrap.js (npm/wasm → globalThis), inlining the .wasm
#   3. spago bundle (PureScript → JS)
#   4. concatenate deps-then-app into dist/index.js
#
# `src = ../app`, so the sandbox source root is the app directory: all
# paths below are relative to that root (no `app/` prefix).
#
# Exposes:
#   - bundle : the static site (dist/{index.html,index.js})
#   - check  : purs-tidy check + spago build + spago test (the
#              playground's slice of `nix flake check`)
{ pkgs
, wasmPkg
}:
let
  # Copy the wasm-pkg outputs into src/vendor/ before bundling, so
  # bootstrap.js (in src/) can `import "./vendor/rdf_shapes_wasm.js"`.
  vendorWasm = ''
    mkdir -p src/vendor
    cp ${wasmPkg}/rdf_shapes_wasm.js src/vendor/
    cp ${wasmPkg}/rdf_shapes_wasm_bg.wasm src/vendor/
    cp ${wasmPkg}/rdf_shapes_wasm.d.ts src/vendor/ 2>/dev/null || true
  '';

  nativeBuildInputs = [
    pkgs.purs
    pkgs.spago-unstable
    pkgs.esbuild
    pkgs.nodejs_22
  ];

  bundle = pkgs.mkSpagoDerivation {
    pname = "rdf-shapes-playground";
    version = "1.0.0";
    src = ../app;
    spagoYaml = ../app/spago.yaml;
    spagoLock = ../app/spago.lock;
    inherit nativeBuildInputs;
    buildPhase = ''
      runHook preBuild

      ${vendorWasm}

      # 1. Bundle npm/wasm deps (bootstrap → globalThis), inlining
      #    the .wasm as binary so evaluation is fully client-side.
      esbuild src/bootstrap.js \
        --bundle \
        --outfile=dist/deps.js \
        --format=iife \
        --platform=browser \
        --loader:.wasm=binary \
        --minify

      # 2. Bundle the PureScript app (Main).
      spago bundle --offline --module Main

      # 3. Concatenate: deps first, then app.
      cat dist/deps.js dist/index.js > dist/bundle.js
      mv dist/bundle.js dist/index.js
      rm dist/deps.js

      runHook postBuild
    '';
    installPhase = ''
      runHook preInstall
      mkdir -p $out
      cp dist/index.html $out/
      cp dist/index.js $out/
      runHook postInstall
    '';
  };

  # The playground's slice of the single Nix gate: format check +
  # compile + unit tests, all in one sandboxed derivation so
  # `nix flake check` actually runs them.
  check = pkgs.mkSpagoDerivation {
    pname = "rdf-shapes-playground-check";
    version = "1.0.0";
    src = ../app;
    spagoYaml = ../app/spago.yaml;
    spagoLock = ../app/spago.lock;
    nativeBuildInputs = nativeBuildInputs ++ [
      pkgs.purs-tidy-bin.purs-tidy-0_10_0
    ];
    buildPhase = ''
      runHook preBuild

      ${vendorWasm}

      # Format gate (org pins purs-tidy 0.10.0).
      purs-tidy check 'src/**/*.purs' 'test/**/*.purs'

      # Compile + unit tests (permalink/machine-path/examples).
      spago build --offline
      spago test --offline

      runHook postBuild
    '';
    installPhase = ''
      mkdir -p $out
      touch $out/ok
    '';
  };
in
{
  inherit bundle check;
}
