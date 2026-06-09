# Feature Specification: SPARQL + SHACL evaluation façade

**Feature Branch**: `feat/sparql-shacl-facade`
**Created**: 2026-06-09
**Status**: Draft
**Issue**: #6 (epic #1)
**Input**: Production façade promoting spikes #3 (SPARQL) and #4 (SHACL) — full
SPARQL 1.1 query and full SHACL Core validation over RDF graphs, exposed through
the wasm surface and the native CLI, shipping the combined reproducible artifact.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Query an RDF graph with SPARQL (Priority: P1)

A consumer (the dashboard in the browser, or an operator at the CLI) provides an
RDF graph and a SPARQL 1.1 query and receives the result set — variable bindings
for SELECT, a boolean for ASK, a graph for CONSTRUCT/DESCRIBE — with each term's
value and datatype preserved.

**Why this priority**: Querying tx-graphs is half the engine's reason to exist
and the first capability the dashboard needs; it is independently useful even
before validation lands.

**Independent Test**: Load a sample treasury Turtle graph, run an existing named
query (e.g. `tx-count`, `asset-flow`), and confirm the returned rows match the
expected values.

**Acceptance Scenarios**:

1. **Given** a valid RDF graph and a SELECT query, **When** the consumer runs the
   query, **Then** they receive an ordered set of rows, each binding variable
   names to typed terms (IRI / literal+datatype / blank node).
2. **Given** a valid graph and an ASK query, **When** run, **Then** the consumer
   receives a single boolean.
3. **Given** a malformed query, **When** run, **Then** the consumer receives a
   structured error identifying the problem, not a crash.

---

### User Story 2 - Validate an RDF graph against SHACL Core shapes (Priority: P1)

A consumer provides an RDF data graph and a SHACL shapes graph and receives a
conformance report: an overall conforms flag plus, for each violation, the focus
node, the offending value/path, the constraint component that failed, a message,
and a severity.

**Why this priority**: Validation is the other half of the engine and the basis
for replacing the Jena `shacl` CLI; equally critical to querying.

**Independent Test**: Validate the existing `history-entry.shacl.ttl` against a
conforming graph (expect conforms) and a deliberately violating graph (expect the
specific violations), and confirm the report content.

**Acceptance Scenarios**:

1. **Given** a data graph that satisfies the shapes, **When** validated, **Then**
   the report conforms with zero violations.
2. **Given** a data graph violating a cardinality, datatype, and node-kind
   constraint, **When** validated, **Then** the report does not conform and lists
   each violation with its focus node, failed constraint component, and message.
3. **Given** shapes that use constraints outside SHACL Core (e.g. SPARQL-based),
   **When** validated, **Then** the engine reports those as unsupported rather
   than silently ignoring them or crashing.

---

### User Story 3 - Same engine in the browser and at the CLI, reproducibly (Priority: P2)

A consumer runs the identical query or validation either in the browser or from
the native CLI and gets equivalent results; the published artifact is
byte-reproducible so an auditor can rebuild it from source and confirm the bytes.

**Why this priority**: Portability and reproducibility are core project goals
(client-side dashboard execution, audit-grade determinism), but they build on the
two evaluation capabilities above.

**Independent Test**: Run the same query/validation through the wasm surface and
the CLI on the same inputs and diff the outputs; build the artifact twice and
compare checksums.

**Acceptance Scenarios**:

1. **Given** the same graph and query/shapes, **When** evaluated via the browser
   surface and via the CLI, **Then** the results are equivalent.
2. **Given** the published artifact, **When** rebuilt from source independently,
   **Then** the checksum is identical.

---

### Edge Cases

- Malformed Turtle in the graph, query, or shapes → structured parse error.
- Query referencing an undefined prefix → structured error, not a panic.
- Empty graph → query returns empty results; validation conforms vacuously.
- Shapes using SHACL-SPARQL or remote/imported graphs → reported as unsupported
  (out of scope; see Assumptions), never a silent pass.
- Very large input → bounded by available memory; no unbounded resource use
  beyond the input size.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The engine MUST accept an in-memory RDF graph (Turtle) plus a
  SPARQL 1.1 query and return the result set, preserving each term's value and
  datatype.
- **FR-002**: The engine MUST support the full SPARQL 1.1 **query** language
  (SELECT, ASK, CONSTRUCT, DESCRIBE and the standard operators/functions).
- **FR-003**: The engine MUST accept an in-memory RDF data graph plus a SHACL
  shapes graph (Turtle) and return a conformance report: an overall conforms flag
  and, per violation, focus node, value/path, failed constraint component,
  message, and severity.
- **FR-004**: The engine MUST support the full **SHACL Core** constraint
  vocabulary.
- **FR-005**: The engine MUST surface malformed input (graph, query, or shapes)
  and unsupported features as structured, actionable errors — never an
  unhandled crash.
- **FR-006**: Both capabilities MUST be reachable through the browser (wasm)
  surface and the native CLI, returning equivalent results for equivalent inputs.
- **FR-007**: The published evaluation artifact MUST be byte-reproducible across
  independent builds.
- **FR-008**: Evaluation MUST run fully in-memory with no network access and no
  required filesystem state.

### Key Entities

- **RDF graph**: a set of triples provided as Turtle; the data under query or
  validation.
- **SPARQL query**: a SPARQL 1.1 query string.
- **Result set**: the query outcome — variable→term bindings (SELECT), a boolean
  (ASK), or a graph (CONSTRUCT/DESCRIBE).
- **SHACL shapes graph**: node/property shapes and their Core constraints, as
  Turtle.
- **Validation report**: overall conformance plus a list of violations, each with
  focus node, path/value, constraint component, message, severity.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Any well-formed SPARQL 1.1 query over a provided graph returns
  correct results (to be evidenced against the W3C SPARQL 1.1 test suite in #7).
- **SC-002**: SHACL Core validation results match the reference engine
  (Apache Jena) on the seed corpus (to be evidenced by the differential harness
  in #7).
- **SC-003**: The same query/validation produces equivalent output in the browser
  and at the CLI for 100% of a shared smoke set.
- **SC-004**: Two independent builds of the published artifact yield an identical
  checksum.
- **SC-005**: A typical treasury graph (the existing named queries and shapes)
  evaluates in well under one second on commodity hardware.

## Assumptions

- Graphs, queries, and shapes are supplied as in-memory Turtle strings; remote
  graph loading and SPARQL endpoints are out of scope (consistent with
  browser/offline operation).
- **SHACL Core only**; SHACL-SPARQL constraints are out of scope for this feature
  and reported as unsupported.
- Conformance against the W3C suites and byte-for-byte differential parity vs
  Jena are delivered by the separate testing feature (#7); this feature delivers
  the evaluation surface and a smoke level of self-checking.
- The browser/server/CLI split is "one source, three targets" (per spike #5):
  this feature delivers the shared core, the wasm surface, and the CLI; the
  server-side native-library consumer is a later `amaru-treasury-tx` ticket.
- The scaffold's reproducible build pipeline (crane + pinned wasm tooling) is
  reused; this feature does not redesign it.
