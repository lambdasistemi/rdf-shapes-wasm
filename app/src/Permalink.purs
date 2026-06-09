-- | Shareable permalinks (US3): a `Session` ↔ a URL hash fragment.
-- |
-- | `encode` serializes the session to JSON, then base64url-encodes it
-- | into an `#s=…` fragment. `decode` reverses that, total (Nothing on
-- | malformed input). Both are pure and unit-tested for round-trip
-- | (Test.Permalink).
module Permalink
  ( encode
  , decode
  , hashPrefix
  ) where

import Prelude

import Data.Argonaut.Core (Json, jsonEmptyObject, stringify)
import Data.Argonaut.Decode (decodeJson, (.:))
import Data.Argonaut.Encode ((:=), (~>))
import Data.Argonaut.Parser (jsonParser)
import Data.Either (hush)
import Data.Maybe (Maybe(..))
import Data.String (Pattern(..), stripPrefix)
import FFI.Base64 (base64urlDecode, base64urlEncode)
import Types (Session)

-- | The fragment prefix carrying an encoded session.
hashPrefix :: String
hashPrefix = "#s="

sessionToJson :: Session -> Json
sessionToJson s =
  "ttl" := s.ttl
    ~> "sparql" := s.sparql
    ~> "shapes" := s.shapes
    ~> jsonEmptyObject

jsonToSession :: Json -> Maybe Session
jsonToSession j = hush do
  o <- decodeJson j
  ttl <- o .: "ttl"
  sparql <- o .: "sparql"
  shapes <- o .: "shapes"
  pure { ttl, sparql, shapes }

-- | Encode a session to a full `#s=…` fragment.
encode :: Session -> String
encode s = hashPrefix <> base64urlEncode (stringify (sessionToJson s))

-- | Decode a fragment (with or without the leading `#s=`) back to a
-- | session. Returns `Nothing` on a malformed or non-session fragment.
decode :: String -> Maybe Session
decode raw = fromBase64 (stripHash raw)
  where
  stripHash s = case stripPrefix (Pattern hashPrefix) s of
    Just rest -> rest
    Nothing -> s
  fromBase64 b64 = do
    decoded <- base64urlDecode b64
    json <- hush (jsonParser decoded)
    jsonToSession json
