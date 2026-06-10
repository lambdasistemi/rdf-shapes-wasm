# Flake checks: the single Nix-first gate. Each entry is a real
# sandboxed crane derivation that runs the underlying tool, so
# `nix flake check` actually exercises clippy, rustfmt, the test
# suite, cargo-deny, and rustdoc — not just "a script was written".
{ pkgs
, craneEnv
, conformance
, src
}:
let
  inherit (craneEnv) craneLib commonArgs cargoArtifacts;
in
{
  # Clippy with warnings denied across all targets and features.
  clippy = craneLib.cargoClippy (
    commonArgs
    // {
      inherit cargoArtifacts;
      cargoClippyExtraArgs =
        "--all-targets --all-features -- --deny warnings";
    }
  );

  # rustfmt check mode.
  fmt = craneLib.cargoFmt { inherit (commonArgs) src; };

  # Unit tests via cargo-nextest.
  nextest = craneLib.cargoNextest (
    commonArgs
    // {
      inherit cargoArtifacts;
    }
  );

  # Supply-chain gate: licenses, advisories, bans, sources.
  deny = craneLib.cargoDeny { inherit (commonArgs) src; };

  # Documentation build with warnings denied.
  doc = craneLib.cargoDoc (
    commonArgs
    // {
      inherit cargoArtifacts;
      env.RUSTDOCFLAGS = "--deny warnings";
    }
  );

  # Corpus-driven differential and expected-result conformance check.
  conformance = import ./conformance.nix {
    inherit pkgs conformance src;
  };
}
