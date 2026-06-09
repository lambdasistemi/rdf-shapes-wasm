-- | Thin FFI over the vendored `rdf-shapes-wasm` engine, seeded on
-- | `globalThis.rdfShapes` by `bootstrap.js`. The app adds NO
-- | evaluation logic of its own (Constitution I) — every SPARQL/SHACL
-- | call goes through the engine.
-- |
-- | The engine throws a JS `Error` on any failure (Turtle load, SPARQL
-- | parse, validation, or an unsupported SHACL-SPARQL construct) and
-- | returns a plain JS object on success. Both bindings catch the throw
-- | and surface it as `Left message`, so callers branch on `Either`.
module FFI.RdfShapes
  ( query
  , validate
  ) where

import Data.Argonaut.Core (Json)
import Data.Either (Either(..))
import Effect (Effect)

foreign import queryImpl
  :: (String -> Either String Json)
  -> (Json -> Either String Json)
  -> String
  -> String
  -> Effect (Either String Json)

foreign import validateImpl
  :: (String -> Either String Json)
  -> (Json -> Either String Json)
  -> String
  -> String
  -> Effect (Either String Json)

-- | Run `sparql` over the graph parsed from `graphTtl`. On success the
-- | `Json` is the engine's tagged `QueryResults` (`kind` =
-- | `solutions` / `boolean` / `graph`).
query :: String -> String -> Effect (Either String Json)
query = queryImpl Left Right

-- | Validate `dataTtl` against `shapesTtl` (SHACL Core). On success the
-- | `Json` is the engine's `ValidationReport` (`conforms` + an array of
-- | `violations`).
validate :: String -> String -> Effect (Either String Json)
validate = validateImpl Left Right
