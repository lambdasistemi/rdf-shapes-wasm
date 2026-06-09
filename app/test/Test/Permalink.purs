-- | Permalink round-trip (US3): encode → decode is the identity on a
-- | session, and a malformed fragment decodes to Nothing.
module Test.Permalink
  ( spec
  ) where

import Prelude

import Data.Maybe (Maybe(..))
import Permalink (decode, encode, hashPrefix)
import Test.Spec (Spec, describe, it)
import Test.Spec.Assertions (shouldEqual)
import Types (Session, emptySession)

sample :: Session
sample =
  { ttl: "@prefix ex: <http://example.org/> .\nex:a ex:b ex:c ."
  , sparql: "SELECT * WHERE { ?s ?p ?o }"
  , shapes: "ex:Shape a sh:NodeShape ."
  }

withUnicode :: Session
withUnicode =
  { ttl: "ex:λ ex:δ \"café — naïve ✓\" ."
  , sparql: "ASK { ?s ?p ?o }"
  , shapes: ""
  }

spec :: Spec Unit
spec = describe "Permalink" do
  it "round-trips a filled session" do
    decode (encode sample) `shouldEqual` Just sample

  it "round-trips an empty session" do
    decode (encode emptySession) `shouldEqual` Just emptySession

  it "round-trips non-ASCII content (UTF-8 safe)" do
    decode (encode withUnicode) `shouldEqual` Just withUnicode

  it "produces a fragment with the #s= prefix" do
    let frag = encode sample
    decode frag `shouldEqual` Just sample
    (frag /= hashPrefix) `shouldEqual` true

  it "decodes a malformed fragment to Nothing" do
    decode "#s=not-valid-base64-@@@" `shouldEqual` (Nothing :: Maybe Session)

  it "decodes a non-session JSON payload to Nothing" do
    -- base64url of {"foo":1} → eyJmb28iOjF9
    decode "#s=eyJmb28iOjF9" `shouldEqual` (Nothing :: Maybe Session)
