#![warn(clippy::non_reentrant_functions)]
#![allow(unused)]

use std::ptr::{null, null_mut};

mod libc {
    pub use std::ffi::{c_char, c_int};
    #[allow(non_camel_case_types)]
    pub type time_t = i64;

    pub fn strtok(s: *mut c_char, t: *const c_char) -> *mut c_char {
        std::ptr::null_mut()
    }
    pub fn strerror(t: c_int) -> *mut c_char {
        std::ptr::null_mut()
    }
    pub fn not_in_config() {}
}

mod libc_ext {
    use std::ffi::c_void;

    pub fn foo() {}
    pub fn bar(_i: i32) {}
    extern "C" {
        pub fn baz(_a: *mut c_void) -> *mut c_void;
    }
}

fn user_defined_libc() {
    unsafe {
        let _ = libc::strtok(null_mut(), null()); // lint
        libc::strtok(null_mut(), null()); // lint
        libc::strerror(0); // lint
        libc::not_in_config(); // don't lint
    }
}

fn user_defined_mod() {
    use libc_ext::bar;

    libc_ext::foo(); // lint
    bar(1); // lint
    let _ = unsafe {
        libc_ext::baz(null_mut()) // don't lint, extern not found
    };
}

fn main() {}
