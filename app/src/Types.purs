-- | The shared, encodable playground state.
module Types
  ( Session
  , emptySession
  ) where

-- | The current inputs: an RDF data graph (Turtle), a SPARQL query, and
-- | a SHACL shapes graph. This is the unit the permalink encodes and
-- | the examples populate.
type Session =
  { ttl :: String
  , sparql :: String
  , shapes :: String
  }

-- | All-empty starting point.
emptySession :: Session
emptySession =
  { ttl: ""
  , sparql: ""
  , shapes: ""
  }
