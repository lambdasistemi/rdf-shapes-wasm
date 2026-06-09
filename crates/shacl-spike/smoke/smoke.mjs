// Headless Node smoke for the SHACL-in-wasm spike (issue #4).
//
// Loads the wasm-bindgen `--target nodejs` module and calls the exported
// `validate(data_ttl, shapes_ttl)` on (a) a conforming graph and (b) a
// violating graph, both validated against the REAL treasury shape
// `history-entry.shacl.ttl`. Prints both reports and asserts the verdict.
//
// Run:  node crates/shacl-spike/smoke/smoke.mjs
// (from the repo root, inside `nix develop`, after building pkg/.)

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { validate } from "./pkg/shacl_spike.js";

const here = dirname(fileURLToPath(import.meta.url));
const td = join(here, "..", "testdata");
const read = (f) => readFileSync(join(td, f), "utf8");

const shapes = read("history-entry.shacl.ttl");
const conforming = read("conforming.ttl");
const violating = read("violating.ttl");

console.log("=== CONFORMING graph vs history-entry.shacl.ttl ===");
const okReport = JSON.parse(validate(conforming, shapes));
console.log(JSON.stringify(okReport, null, 2));

console.log("\n=== VIOLATING graph vs history-entry.shacl.ttl ===");
const badReport = JSON.parse(validate(violating, shapes));
console.log(JSON.stringify(badReport, null, 2));

// Assertions: prove the engine actually discriminates.
let failures = 0;
const check = (name, cond) => {
  console.log(`${cond ? "PASS" : "FAIL"}: ${name}`);
  if (!cond) failures++;
};

check("conforming -> conforms:true", okReport.conforms === true);
check("conforming -> no error", okReport.error == null);
check("conforming -> zero violations", okReport.violation_count === 0);
check("violating -> conforms:false", badReport.conforms === false);
check("violating -> no error", badReport.error == null);
check("violating -> >=1 violation", badReport.violation_count >= 1);

console.log(`\n${failures === 0 ? "SMOKE PASSED" : `SMOKE FAILED (${failures})`}`);
process.exit(failures === 0 ? 0 : 1);
