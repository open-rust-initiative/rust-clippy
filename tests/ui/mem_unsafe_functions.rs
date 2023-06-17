#![feature(c_size_t)]
#![warn(clippy::mem_unsafe_functions)]

use core::ffi::c_size_t as size_t;
use core::ffi::{c_char, c_int, c_size_t, c_void};
use std::ptr::{null, null_mut};

// mock libc crate
mod libc {
    pub use core::ffi::c_size_t as size_t;
    pub use core::ffi::{c_char, c_int, c_void};
    extern "C" {
        pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
        pub fn strncpy(dst: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char;
        pub fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
        pub fn memmove(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
        pub fn memset(dest: *mut c_void, c: c_int, n: size_t) -> *mut c_void;
    }
}

extern "C" {
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    fn strncpy(dst: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    fn memmove(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    fn memset(dest: *mut c_void, c: c_int, n: size_t) -> *mut c_void;
}

unsafe fn with_void_ptrs() {
    let src: *const c_void = null();
    let dst: *mut c_void = null_mut();
    let n: size_t = 1;

    // Should lint these
    let _ = memcpy(dst, src, n);
    let _ = memmove(dst, src, n);
    let _ = memset(dst, 1, n);
    let _ = libc::memcpy(dst, src, n);
    let _ = libc::memmove(dst, src, n);
    use libc::memset;
    let _ = memset(dst, 1, n);
}

unsafe fn with_char_ptrs() {
    let src: *const c_char = null();
    let dst: *mut c_char = null_mut();

    // Should lint these
    let _ = strcpy(dst, src);
    let _ = strncpy(dst, src, 1);
    let _ = libc::strcpy(dst, src);
    use libc::strncpy;
    let _ = strncpy(dst, src, 1);
}

fn main() {
    unsafe {
        with_char_ptrs();
        with_void_ptrs();
    }
}
