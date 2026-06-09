-- | Preloaded example sessions (US2). Each fills the three inputs with
-- | working content lifted from the engine's testdata, so an LLM or a
-- | first-time human starts from something that Runs.
module Examples
  ( Example
  , examples
  ) where

import Prelude

import Types (Session)

-- | A named, runnable session.
type Example =
  { name :: String
  , session :: Session
  }

-- | The treasury-shaped vocabulary prefix shared by the examples.
historyPrefix :: String
historyPrefix =
  "@prefix atx: <https://lambdasistemi.github.io/amaru-treasury-tx/vocab/history#> .\n"

xsdPrefix :: String
xsdPrefix =
  "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n"

cardanoPrefix :: String
cardanoPrefix =
  "@prefix cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#> .\n"

-- | The `HistoryEntryShape`: every required property with its datatype
-- | / node-kind constraint (verbatim from the engine testdata).
historyShape :: String
historyShape =
  historyPrefix
    <> "@prefix sh: <http://www.w3.org/ns/shacl#> .\n"
    <> xsdPrefix
    <> "\n"
    <> "atx:HistoryEntryShape a sh:NodeShape ;\n"
    <> "  sh:targetClass atx:HistoryEntry ;\n"
    <> "  sh:property [ sh:path atx:tx ; sh:minCount 1 ; sh:maxCount 1 ; sh:nodeKind sh:IRI ] ;\n"
    <> "  sh:property [ sh:path atx:txid ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:string ] ;\n"
    <> "  sh:property [ sh:path atx:slot ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:integer ] ;\n"
    <> "  sh:property [ sh:path atx:scope ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:string ] ;\n"
    <> "  sh:property [ sh:path atx:role ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:string ] ;\n"
    <> "  sh:property [ sh:path atx:direction ; sh:minCount 1 ; sh:maxCount 1 ; sh:datatype xsd:string ] .\n"

-- | A well-formed HistoryEntry: every required property present with
-- | the right datatype; `tx` is an IRI. Conforms.
conformingData :: String
conformingData =
  historyPrefix
    <> xsdPrefix
    <> "\n"
    <> "atx:entry-1 a atx:HistoryEntry ;\n"
    <> "  atx:tx <https://lambdasistemi.github.io/amaru-treasury-tx/tx/abc123> ;\n"
    <> "  atx:txid \"abc123\"^^xsd:string ;\n"
    <> "  atx:slot 12345678 ;\n"
    <> "  atx:scope \"operations\"^^xsd:string ;\n"
    <> "  atx:role \"treasurer\"^^xsd:string ;\n"
    <> "  atx:direction \"outgoing\"^^xsd:string .\n"

-- | A malformed HistoryEntry: slot is a string (datatype), tx is a
-- | literal (nodeKind), direction is missing (minCount). Does not
-- | conform — three violations.
violatingData :: String
violatingData =
  historyPrefix
    <> xsdPrefix
    <> "\n"
    <> "atx:entry-bad a atx:HistoryEntry ;\n"
    <> "  atx:tx \"not-an-iri\"^^xsd:string ;\n"
    <> "  atx:txid \"def456\"^^xsd:string ;\n"
    <> "  atx:slot \"not-a-number\"^^xsd:string ;\n"
    <> "  atx:scope \"operations\"^^xsd:string ;\n"
    <> "  atx:role \"treasurer\"^^xsd:string .\n"

-- | A small transaction graph and a COUNT query over it.
txGraph :: String
txGraph =
  cardanoPrefix
    <> "\n"
    <> "<urn:tx:aaaa> a cardano:Transaction .\n"
    <> "<urn:tx:bbbb> a cardano:Transaction .\n"
    <> "<urn:tx:cccc> a cardano:Transaction .\n"
    <> "<urn:out:1>   a cardano:Output .\n"

-- NB: SPARQL uses `PREFIX` (no `@`, no trailing `.`), not the Turtle
-- `@prefix` form used for the data graph above.
txCountQuery :: String
txCountQuery =
  "PREFIX cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#>\n"
    <> "SELECT (COUNT(DISTINCT ?tx) AS ?transactions)\n"
    <> "WHERE { ?tx a cardano:Transaction . }\n"

-- | The preloaded examples shown in the picker.
examples :: Array Example
examples =
  [ { name: "Transaction count (SPARQL)"
    , session: { ttl: txGraph, sparql: txCountQuery, shapes: "" }
    }
  , { name: "History entry — conforming (SHACL)"
    , session: { ttl: conformingData, sparql: "", shapes: historyShape }
    }
  , { name: "History entry — violating (SHACL)"
    , session: { ttl: violatingData, sparql: "", shapes: historyShape }
    }
  ]
