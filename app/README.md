# rdf-shapes playground

A browser SPARQL 1.1 + SHACL Core playground over the
[`rdf-shapes-wasm`](../) engine. Paste an RDF graph (Turtle), a SPARQL
query, and a SHACL shapes graph, press **Run**, and see the typed query
results and the conformance report side by side — entirely client-side,
no server, no network for evaluation.

The app adds **no evaluation logic of its own** (Constitution I): every
query/validate call goes through the Nix-built `wasm-pkg` engine,
vendored into the bundle at build time.

## Live page

Served from the project's GitHub Pages site under
[`/app/`](https://lambdasistemi.github.io/rdf-shapes-wasm/app/).

## Using it

- **Combined run** — fill the three inputs and press Run. An empty query
  or empty shapes pane is simply not run. Engine errors (malformed
  Turtle, SPARQL parse errors, unsupported SHACL-SPARQL) surface in the
  affected pane only; the other pane is unaffected.
- **Examples** — the picker fills the inputs with working content
  (a transaction-count query; a `HistoryEntry` SHACL shape with
  conforming and violating data) lifted from the engine testdata.
- **Copy** — each result pane has a Copy button (pretty-printed JSON).

## Permalink contract (`#s=`)

The current `{ ttl, sparql, shapes }` session is encoded into the URL
hash fragment:

```
#s=<base64url(JSON.stringify({ ttl, sparql, shapes }))>
```

- **Copy link** writes this fragment into the URL (no reload) and copies
  the full URL to the clipboard.
- On load, if a `#s=` fragment is present it is decoded and the inputs
  are restored exactly (round-trip is unit-tested, including non-ASCII).
  A permalink does **not** auto-run.
- The encoding is UTF-8 safe and URL-safe (`-`/`_`, no padding).

## URL-parameter machine path (`?ttl=&sparql=&shapes=`)

For agents driving the tool programmatically, the inputs can be passed
as URL-encoded query parameters:

```
?ttl=<turtle>&sparql=<query>&shapes=<shapes>
```

- On load, if any of `ttl` / `sparql` / `shapes` is present **and** there
  is no `#s=` permalink, the inputs are populated **and the page
  auto-runs**, rendering the result(s).
- Missing keys default to empty (and an empty pane is not run).
- A `#s=` permalink takes precedence over query params.

## Running locally

From the repository root, with the Nix dev shell:

```bash
nix develop -c bash -c 'cd app && spago build'   # compile
nix develop -c bash -c 'cd app && spago test'    # unit tests
nix build .#playground                           # full bundle (dist/)
```

The single gate is `nix flake check`, which includes the `playground`
check (`purs-tidy` + `spago build`/`test`). The bundle build vendors the
`wasm-pkg` engine output and bundles `bootstrap.js` (esbuild, `.wasm`
inlined as binary) with the PureScript app into `dist/{index.html,index.js}`.
