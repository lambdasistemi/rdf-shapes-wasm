-- | SPIKE (#5) Option A crux smoke, compiled under GHC 9.12.3.
--
-- Identical in spirit to spike/option-a-wasmtime/Smoke.hs, but its real
-- purpose is to be COMPILED by ghc9123 via the haskell.nix flake in this
-- directory. Building it at all is the test: it forces `wasmtime-hs` to
-- be built by GHC 9.12.3. If it runs, it also proves the linear-memory
-- string marshalling works on the org default compiler.
{-# LANGUAGE ScopedTypeVariables #-}

module Main (main) where

import Control.Exception (Exception, throwIO)
import Control.Monad (unless)
import Data.Bits (shiftR, (.&.))
import qualified Data.ByteString as B
import qualified Data.ByteString.Char8 as BC
import Data.Int (Int64)
import System.Environment (getArgs)
import System.Exit (exitFailure)
import Wasmtime

main :: IO ()
main = do
    args <- getArgs
    let wasmPath = case args of
            (p : _) -> p
            [] ->
                "target/wasm32-unknown-unknown/release/\
                \rdf_shapes_server_host.wasm"

    putStrLn ("Loading wasm module: " <> wasmPath)
    engine <- newEngine
    store <- newStore engine >>= handleException
    wasm <- wasmFromPath wasmPath
    myModule <- handleException (newModule engine wasm)
    inst <- newInstance store myModule [] >>= handleException

    Just memory <- getExportedMemory store inst "memory"
    Just (versionFun :: IO (Either WasmException Int64)) <-
        getExportedFunction store inst "shapes_version"

    packed <- versionFun >>= handleException
    let ptr = fromIntegral (packed `shiftR` 32) :: Int
        len = fromIntegral (packed .&. 0xFFFFFFFF) :: Int
    mem <- readMemory store memory
    let result = BC.unpack (B.take len (B.drop ptr mem))
    putStrLn ("decoded version string -> " <> show result)

    unless (result == "0.1.0") $ do
        putStrLn "FAIL: unexpected version from the wasm module"
        exitFailure
    putStrLn "OK: wasmtime-hs (ghc9123) read a string out of wasm memory"

handleException :: Exception e => Either e r -> IO r
handleException = either throwIO pure
