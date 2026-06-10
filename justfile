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

# Prepare a release: bump the workspace version, refresh the lockfile, and
# stamp a CHANGELOG section, then commit. Run inside `nix develop`. After
# merging the resulting PR, push the tag to publish:
#   git tag vX.Y.Z && git push origin vX.Y.Z
# The tag triggers the reproducible, Nix-built GitHub release.
release version:
    #!/usr/bin/env bash
    set -euo pipefail
    v='{{version}}'
    [[ "$v" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "version must be X.Y.Z" >&2; exit 1; }
    # Bump [workspace.package].version (the first `version = "..."` in Cargo.toml).
    sed -i -E '0,/^version = ".*"/s//version = "'"$v"'"/' Cargo.toml
    # Sync the lockfile's workspace-member versions (workspace members only).
    cargo update --workspace
    # Insert a new CHANGELOG section above the most recent one.
    today=$(date +%Y-%m-%d)
    awk -v v="$v" -v d="$today" '
      !done && /^## / { print "## " v " - " d "\n\n- _describe changes_\n"; done=1 }
      { print }
    ' CHANGELOG.md > CHANGELOG.md.tmp && mv CHANGELOG.md.tmp CHANGELOG.md
    git add Cargo.toml Cargo.lock CHANGELOG.md
    git commit -m "chore(release): v$v"
    echo
    echo "Committed chore(release): v$v. Edit the CHANGELOG entry, then:"
    echo "  - open + merge a PR for this commit"
    echo "  - git tag v$v && git push origin v$v   # publishes the release"
