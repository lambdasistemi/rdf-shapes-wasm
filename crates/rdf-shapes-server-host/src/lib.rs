//! SPIKE (#5): C-ABI shell over [`rdf_shapes_core`] for the server host.
//!
//! This crate exists only to de-risk how the Haskell server (the
//! amaru-treasury-tx API, which today shells out to Apache Jena) should
//! call the Rust engine. It carries no business logic — every export
//! delegates to `rdf-shapes-core`.
//!
//! It is built TWO ways, from the SAME source, to compare the two host
//! options:
//!
//! * **Option A (wasm-on-server):** `cargo build --target
//!   wasm32-unknown-unknown` produces a plain `.wasm` module with the
//!   C-ABI exports below over linear memory. This is **NOT** a
//!   wasm-bindgen module — wasmtime (and `wasmtime-hs`) can load it,
//!   whereas the browser bundle in `crates/rdf-shapes-wasm` (which uses
//!   `#[wasm_bindgen]` + JS glue) CANNOT be loaded by a WASI runtime.
//!
//! * **Option B (native FFI):** `cargo build` for the host triple
//!   produces a `.so`/`.dylib` exposing the same `extern "C"` symbols,
//!   callable from Haskell via `foreign import ccall`.
//!
//! ## The string ABI (ptr + len over linear memory)
//!
//! Neither plain wasm nor C FFI has a native string type, so we use the
//! canonical contract:
//!
//! * [`shapes_alloc`] / [`shapes_dealloc`] let the host own buffers in
//!   the module's address space (essential for wasm: the host cannot
//!   `malloc` into the module's linear memory itself).
//! * [`shapes_ping`] takes `(in_ptr, in_len)`, writes the UTF-8 result
//!   into a freshly allocated buffer, and returns `(out_ptr, out_len)`
//!   packed into a single `u64` (`ptr << 32 | len`) so the ABI stays
//!   one return value — the lowest-common-denominator that both the
//!   wasm and the C calling conventions handle without out-params.
//! * [`shapes_version`] is the zero-argument smoke (no input
//!   marshalling) used by both host smokes.

//! ### Native vs wasm ABI ergonomics (a finding in itself)
//!
//! The packed-`u64` ptr/len contract is the natural fit for **wasm**:
//! pointers are 32 bits, so `(ptr, len)` packs losslessly into one
//! `u64`, and the host owns linear memory through `shapes_alloc`.
//!
//! For the **native** host that machinery is overkill — a 64-bit
//! pointer does not even fit the high half of the packed `u64`. So the
//! native path additionally exposes NUL-terminated C strings
//! ([`shapes_version_cstr`] / [`shapes_ping_cstr`] / [`shapes_free_cstr`]),
//! which `foreign import ccall` + `peekCString` consume directly. Same
//! source, but each host reaches for the idiom that suits it.

#![allow(unsafe_code)]

use std::alloc::{alloc, dealloc, Layout};
use std::ffi::{c_char, CStr, CString};

/// Allocates `len` bytes inside the module's address space and returns
/// the pointer. The host writes input bytes here before a call, and
/// reads output bytes from here after one. Returns null on a zero-size
/// or overflowing request.
///
/// # Safety
/// The returned pointer must eventually be passed to [`shapes_dealloc`]
/// with the same `len`, and must not be used after that.
#[no_mangle]
pub unsafe extern "C" fn shapes_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    match Layout::from_size_align(len, 1) {
        Ok(layout) => alloc(layout),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a buffer previously returned by [`shapes_alloc`] (or by the
/// out-pointer of [`shapes_ping`]). `len` must match the original
/// allocation size.
///
/// # Safety
/// `ptr` must come from [`shapes_alloc`]/[`shapes_ping`] with the same
/// `len` and must not be freed twice.
#[no_mangle]
pub unsafe extern "C" fn shapes_dealloc(ptr: *mut u8, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len, 1) {
        dealloc(ptr, layout);
    }
}

/// Echoes the UTF-8 input back through [`rdf_shapes_core::ping`].
///
/// Reads `in_len` bytes from `in_ptr`, computes `pong: <input>`, copies
/// the result into a fresh module-owned buffer, and returns the buffer
/// as a packed `u64`: the high 32 bits are the pointer, the low 32 bits
/// the length. The host unpacks it, reads the bytes, then frees the
/// buffer with [`shapes_dealloc`].
///
/// # Safety
/// `in_ptr` must point to `in_len` initialised, valid UTF-8 bytes.
#[no_mangle]
pub unsafe extern "C" fn shapes_ping(in_ptr: *const u8, in_len: usize) -> u64 {
    let input: &str = if in_ptr.is_null() || in_len == 0 {
        ""
    } else {
        let bytes = std::slice::from_raw_parts(in_ptr, in_len);
        std::str::from_utf8(bytes).unwrap_or("<invalid-utf8>")
    };

    let result = rdf_shapes_core::ping(input);
    pack_string(result)
}

/// Returns the core crate version as a packed `(ptr << 32 | len)` `u64`,
/// the same ABI as [`shapes_ping`]. Zero-argument smoke: no input
/// marshalling, so it isolates "can the host call in and read a string
/// out" from "can the host write a string in".
///
/// # Safety
/// The returned pointer must be freed with [`shapes_dealloc`].
#[no_mangle]
pub unsafe extern "C" fn shapes_version() -> u64 {
    pack_string(rdf_shapes_core::version().to_owned())
}

/// Copies `s` into a fresh module-owned buffer and packs the pointer and
/// length into a single `u64`. Returns `0` (null ptr, zero len) for an
/// empty string or on allocation failure.
unsafe fn pack_string(s: String) -> u64 {
    let bytes = s.into_bytes();
    let len = bytes.len();
    if len == 0 {
        return 0;
    }
    let out = shapes_alloc(len);
    if out.is_null() {
        return 0;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, len);
    ((out as u64) << 32) | (len as u64)
}

// --- Native-friendly C-string ABI (Option B) ---------------------------
//
// These are the ergonomic surface for `foreign import ccall`: the host
// gets a `CString` it reads with `peekCString` and returns with
// `shapes_free_cstr`. They are pointless for wasm (the host can't deref
// a raw module pointer without going through linear memory anyway), but
// they make the native FFI smoke a three-liner instead of a memory dance.

/// Returns the core version as a heap-allocated NUL-terminated C string.
/// The caller must free it with [`shapes_free_cstr`].
///
/// # Safety
/// The returned pointer must be freed exactly once with
/// [`shapes_free_cstr`] and not used afterwards.
#[no_mangle]
pub unsafe extern "C" fn shapes_version_cstr() -> *mut c_char {
    into_cstr(rdf_shapes_core::version().to_owned())
}

/// Echoes a NUL-terminated C string back through
/// [`rdf_shapes_core::ping`], returning a fresh C string the caller must
/// free with [`shapes_free_cstr`].
///
/// # Safety
/// `input` must be a valid NUL-terminated C string; the returned pointer
/// must be freed exactly once with [`shapes_free_cstr`].
#[no_mangle]
pub unsafe extern "C" fn shapes_ping_cstr(input: *const c_char) -> *mut c_char {
    let s = if input.is_null() {
        String::new()
    } else {
        CStr::from_ptr(input).to_string_lossy().into_owned()
    };
    into_cstr(rdf_shapes_core::ping(&s))
}

/// Frees a C string previously returned by [`shapes_version_cstr`] or
/// [`shapes_ping_cstr`].
///
/// # Safety
/// `ptr` must come from one of those functions and must not be freed
/// twice.
#[no_mangle]
pub unsafe extern "C" fn shapes_free_cstr(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Builds an owned C string from `s`, leaking it for the caller to free.
/// Replaces any interior NUL so the conversion never fails.
fn into_cstr(s: String) -> *mut c_char {
    let cleaned = s.replace('\0', "");
    // `unwrap` is safe: interior NULs were stripped above.
    CString::new(cleaned).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    // The packed-`u64` ptr/len ABI is wasm-only by construction: it
    // assumes a 32-bit pointer so `(ptr << 32 | len)` is lossless. On a
    // 64-bit host the pointer does not fit the high half, so this test
    // is gated to 32-bit targets. (Under wasm32 it is exercised by the
    // wasmtime host smoke; the native path uses the C-string ABI below.)
    #[cfg(target_pointer_width = "32")]
    #[test]
    fn packed_ping_round_trips() {
        unsafe {
            let input = b"hello";
            let packed = shapes_ping(input.as_ptr(), input.len());
            let ptr = (packed >> 32) as *mut u8;
            let len = (packed & 0xFFFF_FFFF) as usize;
            let out = std::slice::from_raw_parts(ptr, len);
            assert_eq!(out, b"pong: hello");
            shapes_dealloc(ptr, len);
        }
    }

    #[test]
    fn cstr_ping_round_trips() {
        unsafe {
            let input = CString::new("world").unwrap();
            let out = shapes_ping_cstr(input.as_ptr());
            let s = CStr::from_ptr(out).to_str().unwrap().to_owned();
            assert_eq!(s, "pong: world");
            shapes_free_cstr(out);
        }
    }
}
