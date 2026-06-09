# Feature Specification: Browser SPARQL+SHACL playground

**Feature Branch**: `feat/playground`
**Created**: 2026-06-09
**Status**: Draft
**Issue**: #16 (epic #1)
**Input**: A browser app for LLMs and humans to paste RDF Turtle and run SPARQL
1.1 queries and SHACL Core validation against it, fully client-side via the
`rdf-shapes-wasm` engine (#6).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run SPARQL and SHACL over pasted TTL (Priority: P1)

A user (an LLM agent or a human) pastes an RDF graph as Turtle, optionally a
SPARQL query and a SHACL shapes graph, presses Run, and sees — in one view — the
query result set and the SHACL conformance report. Everything runs in the
browser; nothing is sent to a server.

**Why this priority**: This is the product. Without it there is no playground.

**Independent Test**: Open the page, paste a small graph + the `tx-count` query +
the `history-entry` shape, Run, and confirm the result rows and the report appear
and are copyable.

**Acceptance Scenarios**:

1. **Given** a valid graph and SPARQL query, **When** Run, **Then** the typed
   result rows render and can be copied as JSON.
2. **Given** a valid data graph and SHACL shapes, **When** Run, **Then** the
   conformance flag and any violations render and are copyable.
3. **Given** malformed input, **When** Run, **Then** a clear error message shows
   for that pane, and the other pane is unaffected.
4. **Given** no network, **When** Run, **Then** evaluation still works (fully
   client-side).

---

### User Story 2 - Start from preloaded examples (Priority: P2)

The user clicks a sample to populate the graph / query / shapes with working
content (a treasury-shaped graph + a named query; the `history-entry` shape with
conforming and violating data), then edits from there.

**Why this priority**: An LLM or first-time human needs a correct starting point;
examples make the tool self-teaching.

**Independent Test**: Click each example; confirm the inputs fill and Run yields
the expected result/report.

**Acceptance Scenarios**:

1. **Given** the page, **When** an example is selected, **Then** the inputs
   populate with valid, runnable content.

---

### User Story 3 - Share a session as a permalink (Priority: P2)

The current TTL + query + shapes are encoded into the URL so the filled session
is a single link that, when opened, restores the exact inputs.

**Why this priority**: Lets an LLM be handed a link that reproduces a result, or a
human share a repro.

**Acceptance Scenarios**:

1. **Given** filled inputs, **When** the user copies the page link and reopens it,
   **Then** the inputs are restored exactly.

---

### User Story 4 - Drive it from URL params (machine path) (Priority: P3)

An agent opens a URL carrying the inputs as query parameters; the page auto-runs
and renders just the result(s), with no manual interaction.

**Why this priority**: Lets an LLM/agent use the tool programmatically without
operating the UI; lowest priority because the interactive UI already covers the
core need.

**Acceptance Scenarios**:

1. **Given** a URL with `ttl`/`sparql`/`shapes` params, **When** opened, **Then**
   the page evaluates automatically and shows the result(s) prominently.

---

### Edge Cases

- Malformed TTL/query/shapes → per-pane error, no crash, other pane still works.
- Empty query or empty shapes → that pane is simply not run.
- Very large pasted input → bounded by the browser tab's memory.
- Oversized permalink (huge TTL) → fall back gracefully (e.g. warn it's too long
  to encode) rather than produce a broken link.
- Shapes using SHACL-SPARQL → the engine's "unsupported" error is shown, not a
  silent pass.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The app MUST let the user input an RDF graph (Turtle), a SPARQL
  query, and a SHACL shapes graph, and run them with one action.
- **FR-002**: The app MUST evaluate entirely client-side via the engine, with no
  network calls for evaluation.
- **FR-003**: The app MUST render query results (typed bindings / boolean /
  graph) and the SHACL report (conforms + violations) together, in copyable form.
- **FR-004**: The app MUST surface engine errors per pane as readable messages.
- **FR-005**: The app MUST provide preloaded examples that populate the inputs.
- **FR-006**: The app MUST encode the current inputs into a shareable URL and
  restore them when such a URL is opened.
- **FR-007**: The app MUST support a URL-parameter mode that auto-runs from the
  params and renders the result(s).
- **FR-008**: The app MUST be reachable as a live page on the project's site.

### Key Entities

- **Session**: the current TTL graph + SPARQL query + SHACL shapes (the
  encodable/shareable state).
- **Query result**: typed bindings / boolean / graph, as returned by the engine.
- **Validation report**: conforms flag + violations, as returned by the engine.

## Success Criteria *(mandatory)*

- **SC-001**: A user can paste a graph + query and see correct results in under a
  few seconds, with zero setup.
- **SC-002**: A user can paste a graph + shapes and see the correct conformance
  report.
- **SC-003**: A permalink round-trips the inputs exactly (open restores paste).
- **SC-004**: A URL-param link auto-runs and shows the result without any clicks.
- **SC-005**: The page works offline after first load (no evaluation network
  traffic).

## Assumptions

- Inputs are Turtle text pasted into the page; no file upload or remote graph
  fetch (consistent with the engine's in-memory, offline design).
- SHACL Core only (the engine's scope); SHACL-SPARQL surfaces as unsupported.
- The app consumes the existing `rdf-shapes-wasm` `wasm-pkg` (#6); it adds no new
  evaluation logic of its own.
- Deployed on the repo's existing GitHub Pages site (one deployment), under a
  sub-path.
