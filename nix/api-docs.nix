# Reproducible rustdoc HTML for the whole workspace.
#
# Wraps crane's `cargoDoc` over the native dependency artifacts so the
# generated `target/doc` tree is a Nix-built, content-addressed
# derivation. The MkDocs deploy workflow copies this output into
# `site/api/` so the Rust API reference is co-hosted under the
# published documentation site.
#
# Unlike the `doc` *check* (which runs with `--deny warnings` as the
# correctness gate), this PUBLISH build does not deny warnings: the
# site must render even when a doc warning exists. The gate check
# already enforces clean docs, so this stays purely about producing
# HTML.
{ craneEnv }:
let
  inherit (craneEnv) craneLib commonArgs cargoArtifacts;
in
craneLib.cargoDoc (
  commonArgs
  // {
    inherit cargoArtifacts;
    pname = "rdf-shapes-wasm-api-docs";
    # Document the workspace's own crates only — private items
    # included so the reference is complete. No `--deny warnings`
    # here (publish even with doc warnings); the `doc` check gates
    # cleanliness separately.
    cargoDocExtraArgs =
      "--no-deps --workspace --document-private-items";
  }
)
