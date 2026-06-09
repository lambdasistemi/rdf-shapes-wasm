-- | The combined-view Halogen component: three inputs (TTL data, SPARQL
-- | query, SHACL shapes), a single Run action, two copyable result
-- | regions (query results + validation report) with per-pane errors,
-- | an example picker, and a shareable permalink.
-- |
-- | On init it restores a `#s=` permalink, or — if `?ttl=&sparql=&shapes=`
-- | params are present — populates and auto-runs (the machine path).
-- |
-- | The component holds NO evaluation logic; Run delegates to the
-- | `FFI.RdfShapes` bindings over the wasm engine (Constitution I).
module Playground
  ( component
  ) where

import Prelude

import Data.Argonaut.Core (Json)
import Data.Array ((!!))
import Data.Either (Either(..))
import Data.Maybe (Maybe(..))
import Data.String (Pattern(..), contains)
import Effect.Aff.Class (class MonadAff)
import Effect.Class (liftEffect)
import Examples (examples)
import FFI.Clipboard (copyToClipboard)
import FFI.Json (stringifyPretty)
import FFI.Location (getHash, getLocationHref, getQueryParams, setHash)
import FFI.RdfShapes as Engine
import Halogen as H
import Halogen.HTML as HH
import Halogen.HTML.Events as HE
import Halogen.HTML.Properties as HP
import MachinePath (parseParams)
import Permalink as Permalink
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
  , linkCopied :: Boolean
  }

data Action
  = Initialize
  | SetTtl String
  | SetSparql String
  | SetShapes String
  | SelectExample String
  | Run
  | Copy String
  | CopyLink

component :: forall q i o m. MonadAff m => H.Component q i o m
component =
  H.mkComponent
    { initialState
    , render
    , eval: H.mkEval H.defaultEval
        { handleAction = handleAction
        , initialize = Just Initialize
        }
    }

initialState :: forall i. i -> State
initialState _ =
  { session: emptySession
  , queryResult: NotRun
  , validationResult: NotRun
  , linkCopied: false
  }

handleAction
  :: forall o m
   . MonadAff m
  => Action
  -> H.HalogenM State Action () o m Unit
handleAction = case _ of
  Initialize -> do
    hash <- liftEffect getHash
    case Permalink.decode hash of
      -- A permalink takes precedence: restore inputs, do not auto-run.
      Just session -> H.modify_ _ { session = session }
      Nothing -> do
        params <- liftEffect getQueryParams
        case parseParams params of
          -- Machine path: populate AND auto-run, rendering results.
          Just session -> do
            H.modify_ _ { session = session }
            runSession session
          Nothing -> pure unit
  SetTtl s -> setField _ { ttl = s }
  SetSparql s -> setField _ { sparql = s }
  SetShapes s -> setField _ { shapes = s }
  SelectExample name ->
    case findExample name of
      Just session ->
        H.modify_ _
          { session = session
          , queryResult = NotRun
          , validationResult = NotRun
          , linkCopied = false
          }
      Nothing -> pure unit
  Copy text -> liftEffect $ copyToClipboard text
  CopyLink -> do
    { session } <- H.get
    let fragment = Permalink.encode session
    liftEffect $ setHash fragment
    href <- liftEffect getLocationHref
    liftEffect $ copyToClipboard href
    H.modify_ _ { linkCopied = true }
  Run -> do
    { session } <- H.get
    runSession session
  where
  setField f =
    H.modify_ \st -> st { session = f st.session, linkCopied = false }

-- | Run the session through the engine and stash both panes. Empty
-- | query / empty shapes are simply not run (spec edge case).
runSession
  :: forall o m
   . MonadAff m
  => Session
  -> H.HalogenM State Action () o m Unit
runSession session = do
  q <-
    if session.sparql == "" then pure NotRun
    else liftEffect $ toPane <$> Engine.query session.ttl session.sparql
  v <-
    if session.shapes == "" then pure NotRun
    else liftEffect $ toPane <$> Engine.validate session.ttl session.shapes
  H.modify_ _ { queryResult = q, validationResult = v }
  where
  toPane = case _ of
    Left e -> Errored e
    Right j -> Ok j

findExample :: String -> Maybe Session
findExample name =
  _.session <$> arrayFind (\e -> e.name == name) examples

arrayFind :: forall a. (a -> Boolean) -> Array a -> Maybe a
arrayFind p xs = go 0
  where
  go i = case xs !! i of
    Nothing -> Nothing
    Just x -> if p x then Just x else go (i + 1)

render :: forall m. State -> H.ComponentHTML Action () m
render st =
  HH.main_
    [ HH.div [ HP.class_ (H.ClassName "toolbar") ]
        [ HH.button [ HE.onClick \_ -> Run ] [ HH.text "Run" ]
        , examplePicker
        , HH.button
            [ HP.class_ (H.ClassName "secondary")
            , HE.onClick \_ -> CopyLink
            ]
            [ HH.text (if st.linkCopied then "Link copied" else "Copy link") ]
        ]
    , HH.div [ HP.class_ (H.ClassName "grid") ]
        [ field "Data graph (Turtle)" st.session.ttl SetTtl
        , field "SPARQL query" st.session.sparql SetSparql
        ]
    , field "SHACL shapes (Turtle)" st.session.shapes SetShapes
    , HH.div [ HP.class_ (H.ClassName "grid") ]
        [ resultPane "Query results" st.queryResult
        , validationPane st.validationResult
        ]
    ]

examplePicker :: forall m. H.ComponentHTML Action () m
examplePicker =
  HH.select
    [ HE.onValueChange SelectExample ]
    ( [ HH.option [ HP.value "" ] [ HH.text "Load an example…" ] ]
        <> map opt examples
    )
  where
  opt e = HH.option [ HP.value e.name ] [ HH.text e.name ]

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
