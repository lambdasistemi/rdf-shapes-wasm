# Playground

A browser **SPARQL 1.1 + SHACL Core** playground over the
`rdf-shapes-wasm` engine. Paste an RDF graph (Turtle), a SPARQL query,
and a SHACL shapes graph, press **Run**, and see the typed query results
and the conformance report side by side — entirely client-side, with no
server and no network for evaluation.

It adds no evaluation logic of its own: every query/validate call goes
through the same Nix-built WebAssembly engine the CLI and CI use,
vendored into the page at build time.

The app is co-hosted under this site at <code>/app/</code>. Open it in a
full tab via the direct link if the frame below does not load (for
example on a local `mkdocs serve` where `/app/` has not been copied in
yet): <a href="../app/index.html">open the playground</a>.

## Features

- **Combined run** — one Run evaluates the query and the shapes
  together; an empty query or shapes pane is simply not run, and engine
  errors surface per pane.
- **Examples** — a picker fills the inputs with working content
  (a transaction-count query; a `HistoryEntry` SHACL shape with
  conforming and violating data).
- **Permalink** — *Copy link* encodes the current inputs into the URL
  (`#s=…`); opening such a link restores the exact session.
- **Machine path** — opening a URL with `?ttl=&sparql=&shapes=` query
  parameters populates the inputs and auto-runs, for programmatic use by
  an agent.

<iframe
  src="../app/index.html"
  title="rdf-shapes playground"
  style="width:100%;height:85vh;border:0"
  loading="lazy">
</iframe>
