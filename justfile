# Thin wrappers over the Nix gate. `nix flake check` == `just ci` == CI.

# Default recipe: the full gate.
default: ci

# Build the native CLI and core library.
build:
    nix run .#build

# Run the unit test suite (cargo-nextest).
test:
    nix run .#test

# Format the workspace in place.
fmt:
    nix develop -c cargo fmt --all

# Verify formatting without modifying files.
fmt-check:
    nix run .#fmt-check

# Clippy with warnings denied.
clippy:
    nix run .#clippy

# Build the reproducible wasm bundle.
wasm:
    nix run .#wasm

# Build the native C-ABI shared library + header.
ffi:
    nix run .#ffi

# Supply-chain audit (cargo-deny).
deny:
    nix run .#deny

# The single gate: every package + every check, identical to CI.
ci:
    nix run .#ci
