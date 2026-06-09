-- | Machine-path param parsing (US4): build a Session from query
-- | params, triggering only when at least one relevant key is present.
module Test.MachinePath
  ( spec
  ) where

import Prelude

import Data.Maybe (Maybe(..))
import Data.Tuple (Tuple(..))
import MachinePath (parseParams)
import Test.Spec (Spec, describe, it)
import Test.Spec.Assertions (shouldEqual)
import Types (Session)

spec :: Spec Unit
spec = describe "MachinePath.parseParams" do
  it "builds a session from all three params" do
    parseParams
      [ Tuple "ttl" "ex:a ex:b ex:c ."
      , Tuple "sparql" "SELECT * WHERE { ?s ?p ?o }"
      , Tuple "shapes" "ex:S a sh:NodeShape ."
      ]
      `shouldEqual` Just
        { ttl: "ex:a ex:b ex:c ."
        , sparql: "SELECT * WHERE { ?s ?p ?o }"
        , shapes: "ex:S a sh:NodeShape ."
        }

  it "fills missing keys with empty strings" do
    parseParams [ Tuple "sparql" "ASK { ?s ?p ?o }" ]
      `shouldEqual` Just { ttl: "", sparql: "ASK { ?s ?p ?o }", shapes: "" }

  it "returns Nothing when no relevant key is present" do
    parseParams [ Tuple "theme" "dark", Tuple "x" "1" ]
      `shouldEqual` (Nothing :: Maybe Session)

  it "returns Nothing for an empty param list" do
    parseParams [] `shouldEqual` (Nothing :: Maybe Session)

  it "treats an empty value as absent" do
    parseParams [ Tuple "ttl" "" ]
      `shouldEqual` (Nothing :: Maybe Session)
