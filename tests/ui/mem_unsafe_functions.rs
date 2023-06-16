#![feature(c_size_t)]
#![warn(clippy::mem_unsafe_functions)]

use core::ffi::{c_char, c_int, c_size_t, c_void};

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
    fn strncpy(dst: *mut c_char, src: *const c_char, n: c_size_t) -> *mut c_char;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: c_size_t) -> *mut c_void;
    fn memmove(dest: *mut c_void, src: *const c_void, n: c_size_t) -> *mut c_void;
    fn memset(dest: *mut c_void, c: c_int, n: c_size_t) -> *mut c_void;
}

fn main() {}
