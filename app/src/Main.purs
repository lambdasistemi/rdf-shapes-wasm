-- | Application entry point: mount the playground Halogen component.
module Main
  ( main
  ) where

import Prelude

import Effect (Effect)
import Halogen as H
import Halogen.Aff as HA
import Halogen.HTML as HH
import Halogen.VDom.Driver (runUI)

main :: Effect Unit
main = HA.runHalogenAff do
  body <- HA.awaitBody
  void $ runUI placeholder unit body

-- | Minimal mountable placeholder so the scaffold builds and runs.
-- | Replaced in T008 by the `Playground` component.
placeholder :: forall q i o m. H.Component q i o m
placeholder =
  H.mkComponent
    { initialState: const unit
    , render: \_ -> HH.div_ [ HH.text "rdf-shapes playground" ]
    , eval: H.mkEval H.defaultEval
    }
