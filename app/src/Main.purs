-- | Application entry point: mount the playground Halogen component.
module Main
  ( main
  ) where

import Prelude

import Effect (Effect)
import Halogen.Aff as HA
import Halogen.VDom.Driver (runUI)
import Playground as Playground

main :: Effect Unit
main = HA.runHalogenAff do
  body <- HA.awaitBody
  void $ runUI Playground.component unit body
