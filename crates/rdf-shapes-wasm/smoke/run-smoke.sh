#!/usr/bin/env bash
# Parity smoke (US3, T020): run the SAME query and the SAME validation
# through the wasm bundle (Node) and the native CLI, and assert the
# outputs are structurally equivalent.
#
# Run from the repo root inside the Nix dev shell (provides node, the
# pinned wasm-bindgen-cli, wasm-opt, cargo):
#
#   nix develop -c crates/rdf-shapes-wasm/smoke/run-smoke.sh
#
# Exits non-zero on any mismatch or error, so it fails loudly.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
cd "$root"

# 1. Build the reproducible bundle and link it under smoke/pkg.
pkg="$(nix build .#wasm-pkg --no-link --print-out-paths)"
rm -rf "$here/pkg"
mkdir -p "$here/pkg"
cp -f "$pkg"/rdf_shapes_wasm.js "$pkg"/rdf_shapes_wasm_bg.wasm "$here/pkg/"

# 2. Build the native CLI.
cargo build --release -p rdf-shapes-cli
cli="$root/target/release/rdf-shapes"

# canonical-JSON normalizer so key order does not matter.
canon() { node -e '
  const fs = require("fs");
  const v = JSON.parse(fs.readFileSync(0, "utf8"));
  const c = (x) => Array.isArray(x) ? x.map(c)
    : (x && typeof x === "object")
      ? Object.fromEntries(Object.keys(x).sort().map((k) => [k, c(x[k])]))
      : x;
  process.stdout.write(JSON.stringify(c(v)));
'; }

# 3. wasm side: two lines (query result, validation report).
mapfile -t wasm < <(node "$here/smoke.mjs")
wasm_query="${wasm[0]}"
wasm_validate="${wasm[1]}"

# 4. CLI side: same inputs.
cli_query="$("$cli" query --graph "$here/graph.ttl" --query "$here/tx-count.rq" | canon)"
cli_validate="$("$cli" validate --data "$here/data.ttl" --shapes "$here/shapes.ttl" | canon)"

fail=0
if [ "$wasm_query" != "$cli_query" ]; then
  echo "PARITY MISMATCH (query):"
  echo "  wasm: $wasm_query"
  echo "  cli : $cli_query"
  fail=1
fi
if [ "$wasm_validate" != "$cli_validate" ]; then
  echo "PARITY MISMATCH (validate):"
  echo "  wasm: $wasm_validate"
  echo "  cli : $cli_validate"
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  echo "SMOKE FAILED"
  exit 1
fi

echo "SMOKE OK: wasm and CLI agree on query and validate"
echo "  query   : $cli_query"
echo "  validate: $cli_validate"
