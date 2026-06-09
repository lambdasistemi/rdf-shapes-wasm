-- | Native @foreign import ccall@ smoke over @librdf_shapes_ffi@.
--
-- This is the live-boundary proof for issue #19: it links the native
-- C-ABI shared library produced by @nix build .#ffi-lib@ and drives the
-- engine exactly the way the Haskell backend (@amaru-treasury-tx@,
-- replacing Jena) will — through @foreign import ccall@, over C strings,
-- freeing each returned envelope with @rdf_shapes_string_free@.
--
-- Spike #5 proved native ccall from GHC 9.12.3 works (the wasmtime-hs
-- path is blocked on 9.12.3); this is that path made real against the
-- production cdylib. It prints the @{"ok":...}@ envelope for a SPARQL
-- query and a SHACL validation, and an @{"error":...}@ envelope for a
-- malformed query, then exits non-zero if any expectation is unmet.
module Main (main) where

import Control.Monad (unless)
import Data.List (isInfixOf)
import Foreign.C.String (CString, newCString, peekCString)
import Foreign.Ptr (Ptr, nullPtr)
import System.Exit (exitFailure)

-- The three computing functions plus the deallocator, bound exactly as
-- declared in the generated @rdf_shapes.h@.
foreign import ccall unsafe "rdf_shapes_query"
    c_rdf_shapes_query :: CString -> CString -> IO CString

foreign import ccall unsafe "rdf_shapes_validate"
    c_rdf_shapes_validate :: CString -> CString -> IO CString

foreign import ccall unsafe "rdf_shapes_version"
    c_rdf_shapes_version :: IO CString

foreign import ccall unsafe "rdf_shapes_string_free"
    c_rdf_shapes_string_free :: CString -> IO ()

-- | Marshal a returned envelope pointer to a Haskell 'String', then
-- return ownership to the library (the contract: free exactly once).
takeEnvelope :: CString -> IO String
takeEnvelope ptr
    | ptr == nullPtr = pure "<NULL>"
    | otherwise = do
        s <- peekCString ptr
        c_rdf_shapes_string_free ptr
        pure s

graph :: String
graph =
    unlines
        [ "@prefix cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#> ."
        , "<urn:tx:aaaa> a cardano:Transaction ."
        , "<urn:tx:bbbb> a cardano:Transaction ."
        , "<urn:tx:cccc> a cardano:Transaction ."
        , "<urn:out:1>   a cardano:Output ."
        ]

txCount :: String
txCount =
    unlines
        [ "PREFIX cardano: <https://lambdasistemi.github.io/cardano-ledger-rdf/vocab/cardano#>"
        , "SELECT (COUNT(DISTINCT ?tx) AS ?transactions)"
        , "WHERE { ?tx a cardano:Transaction . }"
        ]

shapes :: String
shapes =
    unlines
        [ "@prefix atx: <https://lambdasistemi.github.io/amaru-treasury-tx/vocab/history#> ."
        , "@prefix sh: <http://www.w3.org/ns/shacl#> ."
        , "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> ."
        , "atx:HistoryEntryShape a sh:NodeShape ;"
        , "  sh:targetClass atx:HistoryEntry ;"
        , "  sh:property [ sh:path atx:slot ; sh:minCount 1 ; sh:datatype xsd:integer ] ."
        ]

conforming :: String
conforming =
    unlines
        [ "@prefix atx: <https://lambdasistemi.github.io/amaru-treasury-tx/vocab/history#> ."
        , "@prefix xsd: <http://www.w3.org/2001/XMLSchema#> ."
        , "atx:entry a atx:HistoryEntry ; atx:slot \"42\"^^xsd:integer ."
        ]

-- | Call a one-argument computing function with a marshalled C string.
call1 :: (CString -> IO CString) -> String -> IO String
call1 f a = newCString a >>= f >>= takeEnvelope

-- | Call a two-argument computing function with marshalled C strings.
call2 :: (CString -> CString -> IO CString) -> String -> String -> IO String
call2 f a b = do
    ca <- newCString a
    cb <- newCString b
    f ca cb >>= takeEnvelope

-- | Run the malformed-query case, passing a deliberately bad SPARQL.
callQueryBad :: IO String
callQueryBad = call2 c_rdf_shapes_query graph "SELEKT bogus {"

main :: IO ()
main = do
    ver <- takeEnvelope =<< c_rdf_shapes_version
    putStrLn $ "version : " <> ver

    q <- call2 c_rdf_shapes_query graph txCount
    putStrLn $ "query   : " <> q

    v <- call2 c_rdf_shapes_validate conforming shapes
    putStrLn $ "validate: " <> v

    e <- callQueryBad
    putStrLn $ "error   : " <> e

    -- Assert the live boundary actually carried results, not just
    -- bytes: the query envelope must report 3 transactions, validate
    -- must conform, and the malformed query must surface an error.
    let checks =
            [ ("version ok envelope", "\"ok\":" `isInfixOf` ver)
            , ("query ok envelope", "\"ok\":" `isInfixOf` q)
            , ("query counted 3 transactions", "\"value\":\"3\"" `isInfixOf` q)
            , ("validate ok envelope", "\"ok\":" `isInfixOf` v)
            , ("validate conforms", "\"conforms\":true" `isInfixOf` v)
            , ("malformed query error envelope", "\"error\":" `isInfixOf` e)
            , ("error names a query error", "query error" `isInfixOf` e)
            ]
    let failures = [name | (name, ok) <- checks, not ok]
    unless (null failures) $ do
        mapM_ (\name -> putStrLn $ "FAIL: " <> name) failures
        exitFailure
    putStrLn "SMOKE OK: Haskell ccall reached the native engine"
