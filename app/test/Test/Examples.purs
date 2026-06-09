-- | Each preloaded example (US2) is non-empty and well-formed: it has a
-- | name, a data graph, and at least one of a query or a shapes graph so
-- | that Run produces something.
module Test.Examples
  ( spec
  ) where

import Prelude

import Data.Array (length)
import Data.Foldable (for_)
import Data.String (Pattern(..), contains)
import Examples (examples)
import Test.Spec (Spec, describe, it)
import Test.Spec.Assertions (shouldEqual)

nonEmpty :: String -> Boolean
nonEmpty s = s /= ""

spec :: Spec Unit
spec = describe "Examples" do
  it "ships at least two examples" do
    (length examples >= 2) `shouldEqual` true

  it "each example has a name and a data graph" do
    for_ examples \e -> do
      nonEmpty e.name `shouldEqual` true
      nonEmpty e.session.ttl `shouldEqual` true

  it "each example is runnable (a query or shapes present)" do
    for_ examples \e ->
      (nonEmpty e.session.sparql || nonEmpty e.session.shapes)
        `shouldEqual` true

  it "any SPARQL query uses the SPARQL PREFIX form, not Turtle @prefix" do
    -- Guards the @prefix-vs-PREFIX bug the browser smoke caught: a
    -- Turtle prefix line in a SPARQL query is a parse error.
    for_ examples \e ->
      when (nonEmpty e.session.sparql) do
        contains (Pattern "@prefix") e.session.sparql `shouldEqual` false
