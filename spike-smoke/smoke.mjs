// SPIKE (#3) headless smoke: load the nodejs-target wasm bundle, run the
// REAL treasury `tx-count.rq` named query over a small sample graph, and
// print the result rows. Run with:  node spike-smoke/smoke.mjs
//
// Exits non-zero if the query throws or the count is not the expected 3,
// so the smoke fails loudly rather than printing a wrong answer.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const wasm = await import(join(here, "pkg", "rdf_shapes_wasm.js"));

const turtle = readFileSync(join(here, "sample.ttl"), "utf8");
const sparql = readFileSync(join(here, "tx-count.rq"), "utf8");

console.log("wasm module version():", wasm.version());

const json = wasm.query(turtle, sparql);
const rows = JSON.parse(json);

console.log("query() raw JSON:", json);
console.log("parsed rows:", JSON.stringify(rows, null, 2));

// The aggregate comes back as a SPARQL typed integer literal, e.g.
//   "3"^^<http://www.w3.org/2001/XMLSchema#integer>
const raw = rows[0]?.transactions ?? "";
const n = Number((raw.match(/^"?(\d+)"?/) ?? [])[1]);
if (n !== 3) {
  console.error(`SMOKE FAILED: expected 3 transactions, got ${n} (raw: ${raw})`);
  process.exit(1);
}
console.log(`SMOKE OK: tx-count = ${n}`);
