# Resolves the pinned Rust toolchain from rust-toolchain.toml via
# rust-overlay, so Nix builds use exactly the channel, components, and
# targets declared for local development.
{ pkgs }:
pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml
