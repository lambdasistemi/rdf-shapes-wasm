# Spike #5 — server-side wasm host: VERDICT

**Question.** How should the Haskell server (amaru-treasury-tx's API,
which today shells out to Apache Jena `arq`/`shacl`) call the Rust
SPARQL/SHACL engine, replacing Jena? Two candidates:

- **Option A — wasm on the server** via
  [`wasmtime-hs`](https://github.com/dfinity/wasmtime-hs): run a `.wasm`
  module from Haskell. "One engine everywhere."
- **Option B — native Rust lib**: build the core as a native `cdylib`
  (C ABI) and call it via Haskell FFI (`foreign import ccall`), or
  subprocess the native CLI. wasm only for the browser.

> **Throwaway de-risking spike.** The code here is not meant to ship; it
> exists to measure the two integration paths and produce this verdict.

**Hard constraint:** the org default — and what amaru-treasury-tx targets
(`flake.nix`: `compiler-nix-name = "ghc9123"`, `cabal.project`:
`index-state: 2026-02-17`) — is **GHC 9.12.3**. Every Haskell claim below
is assessed against GHC 9.12.3.

---

## Recommendation: **Option B (native cdylib via Haskell FFI)** for the
## future amaru-treasury-tx server-integration ticket.

Option B builds and runs cleanly; its FFI surface (`foreign import
ccall`) is GHC-version-agnostic and works on 9.12.3 unchanged. Option A
is **blocked on GHC 9.12.3 by two independent walls** (a `primitive`
dependency bound the solver rejects, and — once that is forced open —
a wasmtime C-API struct-layout mismatch that fails to compile against the
only wasmtime nixpkgs ships). Making it build means adopting and
maintaining what is effectively a private fork of an unreleased,
unmaintained binding whose owning team was disbanded. Option A's only
real selling point — "one artifact everywhere" — **does not hold** (the
browser bundle is wasm-bindgen and cannot be loaded server-side). So its
costs buy us nothing the server needs.

---

## Headline finding: GHC 9.12.3 × wasmtime-hs

**wasmtime-hs does NOT solve under GHC 9.12.3 with stock bounds.** Proven
concretely by resolving the cabal plan via haskell.nix with
`compiler-nix-name = "ghc9123"` and `index-state = 2026-02-17` (the exact
amaru-treasury-tx setup), with wasmtime-hs pinned as a
`source-repository-package` (it is not on Hackage). The solver fails:

```
[__0] trying: wasmtime-hs-0.0.0.0 (user goal)
[__1] next goal: primitive (dependency of wasmtime-hs)
[__1] rejecting: primitive-0.9.1.0 (conflict: wasmtime-hs => primitive>=0.7.3 && <0.9)
[__1] trying: primitive-0.8.0.0
[__2] next goal: base (dependency of wasmtime-hs)
[__2] rejecting: base-4.21.1.0/installed-inplace (conflict: primitive => base>=4.9 && <4.20)
[__2] fail (backjumping, conflict set: base, primitive, wasmtime-hs)
Error: [Cabal-7107] Could not resolve dependencies
```

The wall is **`primitive`**, transitively forcing `base`:

- wasmtime-hs requires `primitive >=0.7.3 && <0.9`.
- the only `primitive < 0.9` (i.e. `0.8.x`) requires `base >=4.9 && <4.20`.
- GHC 9.12.3 ships `base-4.21.1.0`, which is **non-reinstallable** (a
  GHC-bundled boot library — you cannot downgrade it).

So `primitive < 0.9` is fundamentally incompatible with GHC 9.12.3: there
is **no valid plan** with wasmtime-hs's published bounds. (The same upper
bound is why wasmtime-hs's own flake builds on **GHC 9.6.6** — its pinned
nixpkgs defaults to 9.6.6 + `primitive 0.8`, where the bound is satisfied.
Confirmed: that flake's nixpkgs `haskellPackages.ghc.version` = `9.6.6`.)

### The `primitive` bound is stale metadata — but punching through it hits a SECOND wall

Adding `allow-newer: wasmtime-hs:primitive` (plus `base`, `vector`,
`transformers`, `bytestring` for the same reason) makes the solver
produce a plan under ghc9123, selecting the GHC-9.12.3-era set:
`primitive-0.9.1.0`, `vector-0.13.2.0`, `transformers-0.6.1.2`,
`bytestring-0.12.2.0`, `base-4.21.1.0`. So the bound is not an API wall —
the Haskell library would type-check against `primitive 0.9`.

**But the build still fails — at the C-API binding layer.** This probe
went all the way: with `allow-newer`, it built the full GHC 9.12.3
closure, built wasmtime + its C-API (via the cmake overlay), and reached
the actual compile of the wasmtime-hs library — where it died:

```
wasmtime-hs-lib> error: In file included from Extern.hsc:4:
wasmtime-hs-lib> Extern.hsc:24:16: error: ‘struct wasmtime_table’ has no member named
                 ‘__private’; did you mean ‘__private1’?
wasmtime-hs-lib> Extern.hsc:31:16: error: ‘struct wasmtime_memory’ has no member named ‘__private’
wasmtime-hs-lib> Extern.hsc:38:16: error: ‘struct wasmtime_global’ has no member named ‘__private’
error: Cannot build '…-wasmtime-hs-lib-wasmtime-hs-0.0.0.0.drv'
compiling dist/build/Bindings/Wasmtime/Extern_hsc_make.c failed (exit code 1)
  …  -I/nix/store/…-wasmtime-43.0.0-dev/include  …
```

This is a **wasmtime C-API version mismatch**, and it is the decisive
practical blocker. `extra-libraries: wasmtime` links the wasmtime C-API,
which stock `pkgs.wasmtime` does not ship — the C-API headers/lib must be
patched in with a `cmake -S crates/c-api` overlay (wasmtime-hs's own flake
does this; this probe copies it). The haskell.nix-era nixpkgs ships
**wasmtime v43**, so the overlay produced v43 headers. wasmtime-hs's
`Bindings/Wasmtime/Extern.hsc` was written for **wasmtime v29** (commit
`fix!: support new wasmtime(v29)`, Feb 2025) and reads struct fields
(`__private` on `wasmtime_table` / `wasmtime_memory` / `wasmtime_global`)
that newer wasmtime renamed to `__private1`. The `.hsc` `hsc_field`
macros therefore fail to compile against the v43 headers.

**So under GHC 9.12.3 there are two independent walls:**

1. **Solver:** `primitive < 0.9` vs the non-reinstallable `base-4.21` of
   ghc9123 — no plan exists with stock bounds. Workaround: a permanent
   `allow-newer` (or fork) that *we* maintain.
2. **C-API ABI:** even with the plan, the binding does not compile against
   the only wasmtime nixpkgs provides (v43); it expects v29 struct
   layouts. Workaround: either source-patch wasmtime-hs's `.hsc` files for
   the new field names, or pin an **old wasmtime v29** into
   amaru-treasury-tx's flake (overriding nixpkgs) and keep it frozen.

**What this means for amaru-treasury-tx:** Option A is not "add a
dependency" — it is "adopt and maintain a fork-in-all-but-name of an
unreleased, unmaintained binding": an `allow-newer` override, a custom
wasmtime-C-API derivation, a `.hsc` patch (or a frozen old-wasmtime pin),
all kept mutually consistent across every GHC/nixpkgs bump. There is no
upstream fix in flight: wasmtime-hs has **zero open PRs**, the bounds and
the v29 bindings are untouched, and the maintaining team is gone (see
maintenance risk below). Each of these is a standing cost Option B simply
does not have.

---

## The other decisive finding: wasm-bindgen ≠ WASI ("one artifact" is a myth)

The browser bundle (`crates/rdf-shapes-wasm`) is built with
**wasm-bindgen**: it emits JS glue and targets a JS host (browser /
Node). A generic WASI runtime like wasmtime **cannot load it** — there
is no JS to run the glue, and the module imports `__wbindgen_*` symbols
the runtime can't supply.

So "wasm on the server" does **not** mean "load the exact browser
`.wasm` in Haskell." For wasmtime you must produce a **separate** build:
a `wasm32-unknown-unknown` (or `wasm32-wasip1`) module with plain
`#[no_mangle] extern "C"` exports over linear memory and **no**
wasm-bindgen. This spike built exactly that (`crates/rdf-shapes-server-host`)
and confirmed:

```
$ wasm-objdump -j Import -x rdf_shapes_server_host.wasm
Section not found: Import          # zero host imports — clean C-ABI module

$ wasm-objdump -j Export -x rdf_shapes_server_host.wasm
 - memory[0] -> "memory"
 - func <shapes_alloc>   -> "shapes_alloc"
 - func <shapes_ping>    -> "shapes_ping"
 - func <shapes_version> -> "shapes_version"
 ...
```

It loads and runs in a bare runtime (no JS host):

```
$ wasmtime --version                       # wasmtime 41.0.0
$ wasmtime run --invoke shapes_version rdf_shapes_server_host.wasm
4785177683296261                           # packed ptr<<32|len -> ptr=1114112 len=5 -> "0.1.0"
```

**Implication for the "one artifact" goal:** it does not survive. The
server needs a distinct build regardless of which option you pick. Both
builds come from the *same Rust source* (`rdf-shapes-core` is the shared
pure crate), but the artifact is not one binary — it is:

- browser: `rdf-shapes-wasm` (wasm-bindgen, `--target web`)
- server, Option A: `rdf-shapes-server-host` compiled to
  `wasm32-unknown-unknown` (C-ABI, no bindgen)
- server, Option B: `rdf-shapes-server-host` compiled to the native
  host triple (`.so`/`.dylib`)

"One *source*, three *targets*" is the honest framing. Since the server
build is separate either way, Option A's purity argument collapses to
"the server target happens to be wasm" — which only adds an interpreter
and a marshalling tax for no portability benefit the server can use.

---

## What was built and what ran

| Artifact | Built on target | Evidence |
|---|---|---|
| `rdf-shapes-server-host` native cdylib (`.so`) | ✅ native host | `cargo build --release` → `librdf_shapes_server_host.so` (332 KB) |
| `rdf-shapes-server-host` wasm (`wasm32-unknown-unknown`) | ✅ wasm | 29 KB `.wasm`, **zero imports** |
| wasm runs in a bare runtime | ✅ wasmtime 41 | `wasmtime run --invoke shapes_version …` decodes to `"0.1.0"` |
| **Option B** Haskell `ccall` → native cdylib | ✅ | `spike/option-b-native-ffi/run.sh` → `pong: haskell` |
| **Option A** solver plan, stock bounds, **ghc9123** | ❌ **blocked** | `[Cabal-7107]` — `primitive<0.9` vs `base-4.21` (see above) |
| **Option A** solver plan, `allow-newer`, **ghc9123** | ✅ resolves | plan selects `primitive-0.9.1.0`, `vector-0.13.2.0`, `transformers-0.6.1.2` |
| **Option A** wasmtime-hs library *compiles* under **ghc9123** | ❌ **fails** | `Extern.hsc`: `struct wasmtime_table has no member __private` (v43 headers, binding expects v29) |

### Option B smoke (native FFI) — runs in seconds, GHC-version-agnostic

```
shapes_version_cstr -> "0.1.0"
shapes_ping_cstr "haskell" -> "pong: haskell"
OK: native FFI round-trip succeeded
```

Marshalling is ~10 lines: `newCString` in, `peekCString` out, one free
call. The Rust side exposes a NUL-terminated C-string ABI
(`shapes_ping_cstr` / `shapes_free_cstr`) that `foreign import ccall`
consumes directly. `foreign import ccall` is a GHC built-in present in
every GHC including 9.12.3 — there is no version-specific risk here.

### Option A smoke (wasmtime-hs) — marshalling, when it builds

There is no string type across the wasm boundary, so the contract is
ptr+len over linear memory. The smoke calls `shapes_version` (returns a
packed `i64` = `ptr<<32 | len`), unpacks the halves, `readMemory`s the
whole linear memory, and slices `[ptr .. ptr+len)`. Passing a string
*in* additionally requires calling the exported `shapes_alloc`, writing
the bytes into linear memory, then calling `shapes_ping ptr len` —
host-managed allocation inside the module's address space. wasmtime-hs's
API supports all of this (`getExportedMemory`, `readMemory`, `writeByte`,
typed `getExportedFunction`), but every call site pays the ptr/len
bookkeeping. (Note also open upstream issue #54: `readMemoryAt` is
broken.)

---

## Measured trade-offs

| Dimension | Option A (wasm + wasmtime-hs) | Option B (native cdylib + FFI) |
|---|---|---|
| **Builds on GHC 9.12.3?** | **No — two walls.** (1) Stock `primitive < 0.9` is unsatisfiable against the non-reinstallable `base-4.21` of ghc9123 — no plan. (2) Forcing the plan with `allow-newer` then fails to *compile*: `Extern.hsc` reads wasmtime-v29 struct fields (`__private`) absent from the v43 C-API nixpkgs ships (`__private1`). Buildable only by carrying both an `allow-newer` override AND a `.hsc` patch or a frozen old-wasmtime pin — i.e. a maintained fork. | **Yes, unchanged.** `foreign import ccall` is a GHC built-in; nothing in the FFI path is version-sensitive. |
| **Build complexity** | Very high. wasmtime-hs is **not on Hackage** (version `0.0.0.0`, changelog still "Released on an unsuspecting world"). Needs: `source-repository-package` + SHA pin, an `allow-newer` override, a custom wasmtime **C-API** derivation (cmake overlay; stock `pkgs.wasmtime` lacks the C-API), a source patch to `Bindings/Wasmtime/Extern.hsc` (or an old-wasmtime-v29 pin overriding nixpkgs), and keeping the Haskell/wasmtime/C-API versions mutually consistent across bumps. | Low. A second crane build target for the existing workspace (`.so`/`.dylib`), then `extra-libraries` + a `foreign import ccall` block. No new Haskell dependency, no C-API patching, no engine. Subprocessing the native CLI is even simpler and is **already the pattern** amaru-treasury-tx uses for Jena. |
| **String / JSON marshalling** | Manual ptr+len over linear memory; host-side `alloc`/`writeByte`/`readMemory` per call. Workable, every boundary crossing bespoke. | Trivial. NUL-terminated `CString` ⇄ `peekCString`/`newCString`; JSON crosses as a serialized string. ~10 lines total. |
| **Performance** | wasm via wasmtime (Cranelift JIT). Fast for wasm, but an interpreter/JIT layer + a linear-memory copy on every call. For SPARQL/SHACL over non-trivial graphs, a real if modest tax. | Native machine code, in-process. No engine, no linear-memory copy. Fastest path. |
| **`wasmtime-hs` maintenance risk** | **High.** 10 stars; last substantive commit **March 2025** ("the T&V team doesn't exist anymore" — owning team disbanded); only 2026 commit is a CI-pin chore. **Zero open PRs.** Pinned to wasmtime **v29** while nixpkgs ships **v43**. Open bugs incl. `readMemoryAt` broken (#54) and a darwin FFI bug (#1; amaru-treasury-tx targets darwin too). No releases, ever. | **None added.** The C ABI is a GHC built-in; the only moving part is our own Rust crate, which we already own. |
| **"Same source → browser + server"** | Source is shared (`rdf-shapes-core`), but the server still needs a **separate non-bindgen wasm build**. The browser `.wasm` is not reusable. | Same: source shared, server gets a separate native build. Identical sharing story, simpler server target. |

---

## Why not Option A, plainly

1. It **does not build on GHC 9.12.3** (our org default). First the
   solver rejects it (`primitive < 0.9` vs the boot `base-4.21`); then,
   with that forced open by `allow-newer`, the C-API binding fails to
   compile against the only wasmtime nixpkgs ships (v43 vs the v29 it
   expects). Making it build means permanently owning an
   `allow-newer` + a `.hsc` patch (or a frozen old-wasmtime pin) — a fork.
2. The browser artifact (wasm-bindgen) is **not** loadable server-side,
   so we build a second wasm module anyway — the artifact is not shared.
3. That leaves Option A as "run our server logic through a JIT'd wasm
   interpreter, reached through an unreleased, bound-stale C-API binding
   from a team that no longer exists, pinned to an old wasmtime, wired
   into haskell.nix by hand-patching the wasmtime C-API into nixpkgs."
4. For that we pay manual linear-memory marshalling on every call and an
   interpreter tax — to gain nothing the server actually needs.

Option B shares the **same Rust source**, builds with a one-line extra
crane target, marshals strings in ~10 lines, runs at native speed, builds
on GHC 9.12.3 with **zero** new third-party Haskell dependencies, and is
GHC-version-robust. amaru-treasury-tx **already subprocesses an external
engine** (Jena) — swapping that for either an in-process native `cdylib`
or a native Rust CLI is a strictly smaller, more familiar integration
than standing up a wasm host.

**Recommendation for the server-integration ticket: Option B.** Prefer
the native `cdylib` + `foreign import ccall` for in-process calls; the
native-CLI-subprocess variant is an acceptable lower-effort fallback that
mirrors the current Jena shell-out. Keep wasm-bindgen strictly for the
browser. Reach for Option A only if a future requirement forces
*sandboxed, untrusted* execution of the engine inside the server — the
one thing in-process native code cannot give you — and even then, budget
for owning the wasmtime-hs bounds/fork.

## How to run the smokes

```bash
# Option B — native cdylib via Haskell FFI (fast, self-contained)
./spike/option-b-native-ffi/run.sh

# Option A — THE ghc9123 crux probe (haskell.nix, compiler-nix-name = ghc9123)
cd spike/option-a-ghc9123
#   WALL 1 — stock bounds: comment out the allow-newer line in cabal.project,
#   then the solver FAILS (primitive<0.9 vs base-4.21):
nix build --no-link -L .#legacyPackages.x86_64-linux.plan-nix
#   WALL 2 — with allow-newer (as committed): the plan resolves, the whole
#   GHC 9.12.3 + wasmtime-v43 closure builds, then the LIBRARY COMPILE FAILS:
#     Extern.hsc: 'struct wasmtime_table' has no member named '__private'
#   (v43 C-API headers vs the v29 layout the binding hardcodes). EXPECTED.
nix build --no-link -L .#wasmtime-hs

# Option A — the original wasmtime-hs smoke (uses wasmtime-hs's OWN dev shell,
#   which is GHC 9.6.6 — NOT the org default; kept for reference only)
git clone https://github.com/dfinity/wasmtime-hs /tmp/wasmtime-hs-probe
WASMTIME_HS=/tmp/wasmtime-hs-probe ./spike/option-a-wasmtime/run.sh

# The underlying wasm module also runs in a bare WASI runtime:
nix develop -c cargo build -p rdf-shapes-server-host \
    --target wasm32-unknown-unknown --release
nix shell nixpkgs#wasmtime -c wasmtime run --invoke shapes_version \
    target/wasm32-unknown-unknown/release/rdf_shapes_server_host.wasm
```
