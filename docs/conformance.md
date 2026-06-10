# Conformance

The conformance gate is the correctness trust anchor for replacing Apache Jena.
It has two inputs:

- a treasury seed corpus run differentially against Apache Jena 5.6.0 (`arq`
  for SPARQL and `shacl` for SHACL);
- curated W3C SPARQL and SHACL Core cases run against committed expected
  results.

The native-only `rdf-shapes-conformance` harness normalizes outputs before
comparison: SELECT results are binding multisets unless the query orders them,
ASK results are booleans, graph results are canonical triple sets, and SHACL
reports compare `sh:conforms` plus violation identity
(`focusNode`, `sourceConstraintComponent`, `resultPath`). Message text is not
part of SHACL equality.

## Known Divergence

`conformance/corpus/shacl/history-entry-violating` intentionally exercises the
copied treasury `sh:or` shape. Apache Jena reports the `sh:OrConstraintComponent`
violation focus node as the invalid data node, `urn:entity:bad`. rudof reports
the focus node as the shape IRI
`https://lambdasistemi.github.io/amaru-treasury-tx/vocab/history#TreasuryEntityShape`.

Jena is correct per SHACL: the focus node is the data node being validated. The
harness carries a narrow allowlist for exactly this case, component, and field;
all other fields and all other cases remain strict. The harness prints the
known-divergence count and issue reference on every run, so this is never a
silent skip.

Tracking issue: <https://github.com/lambdasistemi/rdf-shapes-wasm/issues/25>.

## Running

```bash
just conformance
nix run .#conformance
nix build .#checks.x86_64-linux.conformance
```

The Nix check is network-isolated. W3C and treasury inputs are committed under
`conformance/corpus`; see `conformance/README.md` for provenance and case layout.
