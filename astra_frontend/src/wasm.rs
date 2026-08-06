//! WASM bindings for browser-native compilation.
//!
//! A minimal, dependency-free ABI (no wasm-bindgen glue) so the browser loader
//! at `web/public/pkg/astra_frontend.js` can call the parser directly.
//!
//! Build with:
//!
//! ```text
//! cargo build -p astra_frontend --target wasm32-unknown-unknown --release --features wasm
//! cp target/wasm32-unknown-unknown/release/astra_frontend.wasm \
//!    web/public/pkg/astra_frontend_bg.wasm
//! ```

use crate::parser;
use std::alloc::{alloc, dealloc, Layout};
use std::slice;

/// Allocate `len` bytes of wasm linear memory. Returns a null pointer when the
/// request is empty or the allocation fails.
#[no_mangle]
pub extern "C" fn astra_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::null_mut();
    }
    let layout = match Layout::from_size_align(len, 1) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };
    unsafe { alloc(layout) }
}

/// Free a buffer previously returned by [`astra_alloc`].
#[no_mangle]
pub extern "C" fn astra_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len, 1) {
        unsafe { dealloc(ptr, layout) };
    }
}

/// Parse a Zara program and write a debug description of its AST into
/// caller-provided output slots.
///
/// `input`/`input_len` point at the UTF-8 source. `out_ptr`/`out_len` must be
/// two consecutive `u32` slots in linear memory (8 bytes) that receive the
/// pointer and length of the freshly allocated result string.
///
/// Returns `0` on success (or when the parse produced an error string — the
/// caller treats a non-empty result as the outcome either way).
#[no_mangle]
pub extern "C" fn astra_compile(
    input: *const u8,
    input_len: usize,
    out_ptr: *mut u32,
    out_len: *mut u32,
) -> u32 {
    let code = if input.is_null() || input_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { slice::from_raw_parts(input, input_len) };
        String::from_utf8_lossy(bytes).into_owned()
    };

    let result: String = match parser::parse(&code) {
        Ok(program) => format!("{:#?}", program),
        Err(e) => format!("parse error: {}", e.render()),
    };

    let bytes = result.into_bytes();
    let len = bytes.len();
    let ptr = astra_alloc(len);

    unsafe {
        if ptr.is_null() {
            *out_ptr = 0;
            *out_len = 0;
            return 1;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        *out_ptr = ptr as u32;
        *out_len = len as u32;
    }
    0
}
