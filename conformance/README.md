# Conformance Corpus

This directory is the committed input to `rdf-shapes-conformance`. The Nix
check is network-isolated: adding a case means committing its graph, query or
shapes, and expected result.

## Layout

- `corpus/sparql/<case>/{graph.ttl,query.rq,meta.json}`: seed differential
  cases. The harness runs `rdf-shapes-core::query` and Apache Jena `arq` and
  compares semantic results.
- `corpus/shacl/<case>/{data.ttl,shapes.ttl,meta.json}`: seed differential
  cases. The harness runs `rdf-shapes-core::validate` and Apache Jena `shacl`.
- `corpus/w3c/sparql/<case>/{graph.ttl,query.rq,expected.srj|expected.nt}`:
  committed expected-result cases.
- `corpus/w3c/shacl/<case>/{data.ttl,shapes.ttl,expected.ttl}`: committed
  expected SHACL report cases.
- `corpus/w3c/skips/*.md`: explicit unsupported-scope skips.

## Provenance

Treasury queries are copied from
`/code/amaru-treasury-tx/lib/Amaru/Treasury/History/queries/*.rq`. Treasury
SHACL shapes are copied from
`/code/amaru-treasury-tx/lib/Amaru/Treasury/History/shapes/*.shacl.ttl`.
Sample graphs in this repository are intentionally small, handwritten fixtures
that exercise each copied query or shape.

W3C SPARQL cases are curated from `w3c/rdf-tests` commit
`f25dbc092c654d792974848e81bb519d7328f0e8`. W3C SHACL Core cases are curated
from `w3c/data-shapes` commit
`b6e73695d6196f33d7ce3ba47094a10fbc298e65`. Per-case `meta.json` files record
the exact upstream path. `construct-ident/expected.nt` is the official
`result-ident.ttl` normalized to N-Triples for the graph comparator.

The copied W3C SHACL cases use relative manifest IRIs upstream (`<>`,
`<minCount-001>`, and similar). The committed copies rewrite those manifest IRIs
to absolute `http://example.com/w3c/shacl/...` IRIs because
`rdf-shapes-core::validate` intentionally accepts in-memory Turtle strings
without a filesystem base IRI.

## Known Divergence

`corpus/shacl/history-entry-violating` has one narrow allowlisted engine
divergence: for the copied treasury `sh:or` constraint, Jena reports the
`sh:OrConstraintComponent` focus node as the invalid data node
`urn:entity:bad`, while rudof reports the shape IRI. Jena is correct per SHACL.
The harness allowlist is keyed exactly to this case/component/field and prints
the known-divergence count plus issue reference on every run.

Tracking issue: https://github.com/lambdasistemi/rdf-shapes-wasm/issues/25

## Adding A Case

1. Add a new directory under the appropriate corpus family.
2. Commit all input files and expected results; do not fetch at check time.
3. Add `meta.json` with source/provenance and `ordered: true` for SELECT cases
   whose query has ordering semantics.
4. Run `just conformance` locally. For expected-result cases, perturb the
   expected file once and confirm the command fails before reverting.
