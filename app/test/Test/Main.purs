-- | Unit test entry point for the playground's pure logic: permalink
-- | round-trip, machine-path parsing, and example well-formedness.
module Test.Main
  ( main
  ) where

import Prelude

import Effect (Effect)
import Effect.Aff (launchAff_)
import Test.Examples as Examples
import Test.MachinePath as MachinePath
import Test.Permalink as Permalink
import Test.Spec.Reporter (consoleReporter)
import Test.Spec.Runner (runSpec)

main :: Effect Unit
main = launchAff_ $ runSpec [ consoleReporter ] do
  Permalink.spec
  MachinePath.spec
  Examples.spec
