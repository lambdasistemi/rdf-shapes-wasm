-- | Read and mutate the page URL: the hash fragment (permalink) and the
-- | query params (machine path). The parsing/encoding logic lives in the
-- | pure `Permalink` / `MachinePath` modules; this is just the shell.
module FFI.Location
  ( getHash
  , setHash
  , getLocationHref
  , getQueryParams
  ) where

import Prelude

import Data.Tuple (Tuple(..))
import Effect (Effect)

-- | `window.location.hash` (includes the leading `#`, or "").
foreign import getHash :: Effect String

-- | The full current URL (`window.location.href`), used as the
-- | copyable permalink after `setHash`.
foreign import getLocationHref :: Effect String

-- | Replace the URL fragment without a reload (`history.replaceState`).
foreign import setHash :: String -> Effect Unit

foreign import getQueryParamsImpl
  :: (String -> String -> Tuple String String)
  -> Effect (Array (Tuple String String))

-- | The decoded query params as key/value pairs.
getQueryParams :: Effect (Array (Tuple String String))
getQueryParams = getQueryParamsImpl Tuple
