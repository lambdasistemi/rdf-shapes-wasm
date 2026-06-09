-- | SPIKE (#5) Option A smoke: load the plain (NON-wasm-bindgen) wasm
-- module from Haskell via `wasmtime-hs` and call its C-ABI export,
-- reading the result string out of the module's linear memory.
--
-- This loads `target/wasm32-unknown-unknown/release/rdf_shapes_server_host.wasm`
-- — the WASI-loadable build of `crates/rdf-shapes-server-host`, NOT the
-- `crates/rdf-shapes-wasm` browser bundle (that one is wasm-bindgen + JS
-- glue and cannot be instantiated by a bare WASI runtime).
--
-- It demonstrates the string-out-of-linear-memory marshalling that
-- Option A requires:
--
--   1. call `shapes_version` (no args) -> packed i64 (ptr<<32 | len)
--   2. unpack ptr (high 32 bits) and len (low 32 bits)
--   3. `readMemory` the whole linear memory, slice [ptr .. ptr+len)
--   4. decode UTF-8
--
-- Passing a string IN would additionally call the exported
-- `shapes_alloc`, `writeByte` the input bytes into linear memory at the
-- returned offset, then call `shapes_ping ptr len`. This smoke does the
-- read half (the version call) to keep the proof minimal; the write
-- half uses the same `readMemory`/`writeByte` primitives shown in
-- wasmtime-hs's own `test/memory.hs`.
{-# LANGUAGE ScopedTypeVariables #-}

module Main (main) where

import Control.Exception (Exception, throwIO)
import Control.Monad (unless)
import Data.Bits (shiftR, (.&.))
import qualified Data.ByteString as B
import qualified Data.ByteString.Char8 as BC
import Data.Int (Int32, Int64)
import System.Environment (getArgs)
import System.Exit (exitFailure)
import Wasmtime

main :: IO ()
main = do
    args <- getArgs
    let wasmPath = case args of
            (p : _) -> p
            [] -> "target/wasm32-unknown-unknown/release/rdf_shapes_server_host.wasm"

    putStrLn ("Loading wasm module: " <> wasmPath)
    engine <- newEngine
    store <- newStore engine >>= handleException
    wasm <- wasmFromPath wasmPath
    myModule <- handleException (newModule engine wasm)
    inst <- newInstance store myModule [] >>= handleException

    -- The exported linear memory + the zero-arg C-ABI function.
    Just memory <- getExportedMemory store inst "memory"
    Just (versionFun :: IO (Either WasmException Int64)) <-
        getExportedFunction store inst "shapes_version"

    packed <- versionFun >>= handleException
    let ptr = fromIntegral (packed `shiftR` 32) :: Int
        len = fromIntegral (packed .&. 0xFFFFFFFF) :: Int
    putStrLn ("shapes_version -> packed=" <> show packed
                  <> " ptr=" <> show ptr <> " len=" <> show len)

    mem <- readMemory store memory
    let result = BC.unpack (B.take len (B.drop ptr mem))
    putStrLn ("decoded version string -> " <> show result)

    unless (result == "0.1.0") $ do
        putStrLn "FAIL: unexpected version from the wasm module"
        exitFailure
    putStrLn "OK: wasmtime-hs read a string out of wasm linear memory"

handleException :: Exception e => Either e r -> IO r
handleException = either throwIO pure
