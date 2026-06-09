-- | The combined-view Halogen component: three inputs (TTL data, SPARQL
-- | query, SHACL shapes), a single Run action, and two copyable result
-- | regions (query results + validation report) with per-pane errors.
-- |
-- | The component holds NO evaluation logic; Run delegates to the
-- | `FFI.RdfShapes` bindings over the wasm engine (Constitution I).
module Playground
  ( component
  ) where

import Prelude

import Data.Argonaut.Core (Json)
import Data.Either (Either(..))
import Data.Maybe (Maybe(..))
import Data.String (Pattern(..), contains)
import Effect.Aff.Class (class MonadAff)
import Effect.Class (liftEffect)
import FFI.Clipboard (copyToClipboard)
import FFI.Json (stringifyPretty)
import FFI.RdfShapes as Engine
import Halogen as H
import Halogen.HTML as HH
import Halogen.HTML.Events as HE
import Halogen.HTML.Properties as HP
import Types (Session, emptySession)

-- | A pane's outcome after a Run: not yet run, an engine error, or a
-- | JSON result.
data PaneResult
  = NotRun
  | Errored String
  | Ok Json

type State =
  { session :: Session
  , queryResult :: PaneResult
  , validationResult :: PaneResult
  }

data Action
  = SetTtl String
  | SetSparql String
  | SetShapes String
  | Run
  | Copy String

component :: forall q i o m. MonadAff m => H.Component q i o m
component =
  H.mkComponent
    { initialState
    , render
    , eval: H.mkEval H.defaultEval { handleAction = handleAction }
    }

initialState :: forall i. i -> State
initialState _ =
  { session: emptySession
  , queryResult: NotRun
  , validationResult: NotRun
  }

handleAction
  :: forall o m
   . MonadAff m
  => Action
  -> H.HalogenM State Action () o m Unit
handleAction = case _ of
  SetTtl s -> H.modify_ \st -> st { session = st.session { ttl = s } }
  SetSparql s -> H.modify_ \st -> st { session = st.session { sparql = s } }
  SetShapes s -> H.modify_ \st -> st { session = st.session { shapes = s } }
  Copy text -> liftEffect $ copyToClipboard text
  Run -> do
    { session } <- H.get
    -- Empty query / empty shapes simply aren't run (spec edge case).
    q <-
      if session.sparql == "" then pure NotRun
      else liftEffect $ toPane <$> Engine.query session.ttl session.sparql
    v <-
      if session.shapes == "" then pure NotRun
      else liftEffect $ toPane <$> Engine.validate session.ttl session.shapes
    H.modify_ _ { queryResult = q, validationResult = v }
  where
  toPane :: Either String Json -> PaneResult
  toPane = case _ of
    Left e -> Errored e
    Right j -> Ok j

render :: forall m. State -> H.ComponentHTML Action () m
render st =
  HH.main_
    [ HH.div [ HP.class_ (H.ClassName "grid") ]
        [ field "Data graph (Turtle)" st.session.ttl SetTtl
        , field "SPARQL query" st.session.sparql SetSparql
        ]
    , field "SHACL shapes (Turtle)" st.session.shapes SetShapes
    , HH.div [ HP.class_ (H.ClassName "toolbar") ]
        [ HH.button [ HE.onClick \_ -> Run ] [ HH.text "Run" ] ]
    , HH.div [ HP.class_ (H.ClassName "grid") ]
        [ resultPane "Query results" st.queryResult
        , validationPane st.validationResult
        ]
    ]

field
  :: forall m
   . String
  -> String
  -> (String -> Action)
  -> H.ComponentHTML Action () m
field lbl value onInput =
  HH.div [ HP.class_ (H.ClassName "field") ]
    [ HH.label_ [ HH.text lbl ]
    , HH.textarea
        [ HP.value value
        , HE.onValueInput onInput
        , HP.spellcheck false
        ]
    ]

-- | A generic result pane: header with a Copy button, then the
-- | pretty-printed JSON or a per-pane error.
resultPane :: forall m. String -> PaneResult -> H.ComponentHTML Action () m
resultPane title = case _ of
  NotRun ->
    paneShell title Nothing
      [ HH.p [ HP.class_ (H.ClassName "hint") ] [ HH.text "Not run." ] ]
  Errored e ->
    paneShell title Nothing
      [ HH.div [ HP.class_ (H.ClassName "error") ] [ HH.text e ] ]
  Ok j ->
    let
      pretty = stringifyPretty j
    in
      paneShell title (Just pretty) [ HH.pre_ [ HH.text pretty ] ]

-- | The validation pane adds a conforms / non-conforming badge on top
-- | of the generic JSON rendering.
validationPane :: forall m. PaneResult -> H.ComponentHTML Action () m
validationPane = case _ of
  NotRun ->
    paneShell "SHACL report" Nothing
      [ HH.p [ HP.class_ (H.ClassName "hint") ] [ HH.text "Not run." ] ]
  Errored e ->
    paneShell "SHACL report" Nothing
      [ HH.div [ HP.class_ (H.ClassName "error") ] [ HH.text e ] ]
  Ok j ->
    let
      pretty = stringifyPretty j
    in
      paneShell "SHACL report" (Just pretty)
        [ conformsBadge pretty, HH.pre_ [ HH.text pretty ] ]

-- | Render a conforms / does-not-conform badge by reading the flag out
-- | of the pretty JSON text (the report always carries `conforms`).
conformsBadge :: forall m. String -> H.ComponentHTML Action () m
conformsBadge pretty =
  if contains (Pattern "\"conforms\": true") pretty then
    HH.span [ HP.class_ (H.ClassName "badge ok") ] [ HH.text "conforms" ]
  else
    HH.span [ HP.class_ (H.ClassName "badge bad") ]
      [ HH.text "does not conform" ]

paneShell
  :: forall m
   . String
  -> Maybe String
  -> Array (H.ComponentHTML Action () m)
  -> H.ComponentHTML Action () m
paneShell title mCopy body =
  HH.div [ HP.class_ (H.ClassName "result") ]
    [ HH.div [ HP.class_ (H.ClassName "pane-head") ]
        ([ HH.h2_ [ HH.text title ] ] <> copyBtn mCopy)
    , HH.div_ body
    ]
  where
  copyBtn = case _ of
    Nothing -> []
    Just text ->
      [ HH.button
          [ HP.classes [ H.ClassName "secondary", H.ClassName "copy-btn" ]
          , HE.onClick \_ -> Copy text
          ]
          [ HH.text "Copy" ]
      ]
