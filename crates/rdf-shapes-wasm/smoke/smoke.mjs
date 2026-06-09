// Headless Node smoke for the production wasm bundle (web target).
//
// Loads pkg/rdf_shapes_wasm.js, instantiates the .wasm from bytes, and
// runs query() over graph.ttl + tx-count.rq and validate() over
// data.ttl + shapes.ttl. Prints each result as canonical JSON on its
// own line so run-smoke.sh can diff it against the native CLI's output
// for the same inputs (US3 / SC-003 parity).
//
// Exits non-zero on any thrown error.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const pkg = join(here, "pkg");

const mod = await import(join(pkg, "rdf_shapes_wasm.js"));
const wasmBytes = readFileSync(join(pkg, "rdf_shapes_wasm_bg.wasm"));
await mod.default({ module_or_path: wasmBytes });

// Canonical stringify: sort object keys so the wasm and CLI JSON
// compare structurally regardless of serializer key order.
function canon(value) {
  if (Array.isArray(value)) return value.map(canon);
  if (value && typeof value === "object") {
    const out = {};
    for (const k of Object.keys(value).sort()) out[k] = canon(value[k]);
    return out;
  }
  return value;
}
const line = (v) => JSON.stringify(canon(v));

const graph = readFileSync(join(here, "graph.ttl"), "utf8");
const sparql = readFileSync(join(here, "tx-count.rq"), "utf8");
const data = readFileSync(join(here, "data.ttl"), "utf8");
const shapes = readFileSync(join(here, "shapes.ttl"), "utf8");

const queryResult = mod.query(graph, sparql);
const validateResult = mod.validate(data, shapes);

// Two lines: the query result, then the validation report.
process.stdout.write(line(queryResult) + "\n");
process.stdout.write(line(validateResult) + "\n");
