-- | URL-parameter machine path (US4): build a `Session` from query
-- | params `?ttl=&sparql=&shapes=`. Pure and unit-tested
-- | (Test.MachinePath); the `FFI.Location` shell supplies the params
-- | and the app auto-runs when this returns `Just`.
module MachinePath
  ( parseParams
  ) where

import Prelude

import Data.Foldable (any)
import Data.Maybe (Maybe(..), fromMaybe)
import Data.Tuple (Tuple)
import Foreign.Object (Object)
import Foreign.Object as Object
import Types (Session)

-- | Build a session from decoded query params. Returns `Just` iff at
-- | least one of `ttl` / `sparql` / `shapes` carries a non-empty value
-- | (so a bare URL with unrelated params does not trigger the machine
-- | path). Missing keys default to empty strings.
parseParams :: Array (Tuple String String) -> Maybe Session
parseParams params =
  if any present [ "ttl", "sparql", "shapes" ] then
    Just
      { ttl: lookup "ttl"
      , sparql: lookup "sparql"
      , shapes: lookup "shapes"
      }
  else
    Nothing
  where
  obj :: Object String
  obj = Object.fromFoldable params

  lookup :: String -> String
  lookup k = fromMaybe "" (Object.lookup k obj)

  present :: String -> Boolean
  present k = case Object.lookup k obj of
    Just v -> v /= ""
    Nothing -> false
