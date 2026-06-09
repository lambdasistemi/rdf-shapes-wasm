-- | SPIKE (#5) Option B smoke: call the native Rust cdylib from Haskell
-- via `foreign import ccall`.
--
-- This links against `librdf_shapes_server_host.so` (the native build of
-- `crates/rdf-shapes-server-host`) and calls its `extern "C"` exports.
-- It uses the ergonomic NUL-terminated C-string ABI: the Rust side
-- returns a `CString`, Haskell reads it with `peekCString`, then hands
-- it back for the Rust side to free.
--
-- Run it with `./run.sh` in this directory (builds the cdylib, then
-- compiles + runs this smoke against it). No wasm runtime, no host
-- engine — just the C FFI that GHC and every other language already
-- speaks.
module Main (main) where

import Control.Monad (unless)
import Foreign.C.String (CString, newCString, peekCString)
import Foreign.Ptr (Ptr, nullPtr)
import System.Exit (exitFailure)

-- The three native exports from rdf-shapes-server-host's C-string ABI.
foreign import ccall unsafe "shapes_version_cstr"
    c_shapes_version_cstr :: IO CString

foreign import ccall unsafe "shapes_ping_cstr"
    c_shapes_ping_cstr :: CString -> IO CString

foreign import ccall unsafe "shapes_free_cstr"
    c_shapes_free_cstr :: CString -> IO ()

-- | Read a Rust-owned C string into a Haskell 'String', then free it on
-- the Rust side. This is the whole marshalling story for Option B.
takeRustString :: CString -> IO String
takeRustString ptr
    | ptr == nullPtr = pure ""
    | otherwise = do
        s <- peekCString ptr
        c_shapes_free_cstr ptr
        pure s

main :: IO ()
main = do
    -- Zero-argument smoke: read a string out of the native lib.
    version <- c_shapes_version_cstr >>= takeRustString
    putStrLn ("shapes_version_cstr -> " <> show version)

    -- Round-trip a string IN and OUT.
    input <- newCString "haskell"
    pong <- c_shapes_ping_cstr input >>= takeRustString
    putStrLn ("shapes_ping_cstr \"haskell\" -> " <> show pong)

    let ok = version == "0.1.0" && pong == "pong: haskell"
    unless ok $ do
        putStrLn "FAIL: unexpected results from the native cdylib"
        exitFailure
    putStrLn "OK: native FFI round-trip succeeded"
