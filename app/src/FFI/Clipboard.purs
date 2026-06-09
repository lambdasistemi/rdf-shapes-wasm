-- | Write text to the system clipboard (the result panes' Copy action).
module FFI.Clipboard
  ( copyToClipboard
  ) where

import Prelude

import Effect (Effect)

foreign import copyToClipboard :: String -> Effect Unit
