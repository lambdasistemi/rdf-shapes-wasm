-- | Small JSON helpers not covered by argonaut: pretty-printing for the
-- | copyable result regions, and a parser for the URL-param machine
-- | path / examples.
module FFI.Json
  ( stringifyPretty
  , parseJson
  ) where

import Data.Argonaut.Core (Json)

-- | Indented `JSON.stringify(j, null, 2)` for human-readable, copyable
-- | result panes.
foreign import stringifyPretty :: Json -> String

-- | `JSON.parse`. Total here only because callers feed it strings the
-- | engine or our own encoder produced; malformed input throws.
foreign import parseJson :: String -> Json
