-- | URL-safe Base64 over UTF-8, pure and environment-agnostic (browser
-- | `btoa`/`atob` or Node `Buffer`). Used by `Permalink`.
module FFI.Base64
  ( base64urlEncode
  , base64urlDecode
  ) where

import Data.Maybe (Maybe(..))

-- | Encode a UTF-8 string to URL-safe Base64 (no padding).
foreign import base64urlEncode :: String -> String

foreign import base64urlDecodeImpl
  :: Maybe String
  -> (String -> Maybe String)
  -> String
  -> Maybe String

-- | Decode URL-safe Base64 back to a UTF-8 string; `Nothing` on
-- | malformed input.
base64urlDecode :: String -> Maybe String
base64urlDecode = base64urlDecodeImpl Nothing Just
