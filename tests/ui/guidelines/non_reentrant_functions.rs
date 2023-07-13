#![warn(clippy::non_reentrant_functions)]
#![allow(unused)]
#![allow(non_camel_case_types)]
#![feature(rustc_private)]

use std::ffi::{CString, c_int, c_char};
use std::ptr::null_mut;

// Mocked `tm` type. (`libc::tm` is a unix specific type)
#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
}
pub type time_t = i64;

mod with_libc_and_extern {
    // Avoid using any platform dependent api in libc
    // such as `libc::localtime` or `libc::tm`
    extern crate libc;

    extern "C" {
        fn localtime(time: *const time_t) -> *mut tm;
        fn setlocale(category: c_int, locale: *const c_char) -> *mut c_char;
    }
    
    use libc::{time_t, strtok};
    use super::*;

    fn test_libc_strtok() {
        let string = CString::new("welcome-to-rust").unwrap();
        let string = string.as_ptr() as *mut libc::c_char;
        let delim = CString::new(" - ").unwrap();
        let delim = delim.as_ptr();
    
        unsafe {
            let _ = libc::strtok(string, delim); // lint
            let _ = strtok(string, delim); // lint
            let _ = strtok(std::ptr::null_mut(), delim); // lint
        }
    }

    fn test_extern_fns() {
        let time = &123456_i64 as *const time_t;
        let loc = "".as_ptr() as *const c_char;
    
        unsafe {
            let _ = localtime(time); // lint
            localtime(time); // lint
            let a = setlocale(0, loc); // lint
            let _ = libc::strtok(setlocale(0, loc), "".as_ptr() as *const c_char); // lint both `set_locale` and `strtok`
        }
    }
}

mod fake_libc {
    mod libc {
        pub use std::ffi::{c_char, c_int};
        #[allow(non_camel_case_types)]
        pub type time_t = i64;

        extern "C" {
            pub fn strtok(s: *mut c_char, t: *const c_char) -> *mut c_char;
            pub fn stderr(t: c_int) -> *mut c_char;
        }
    }

    // Possible FN: Don't lint user defined mod even if they are called libc,
    // because it's hard to determain the exact properties of such functions.
    fn user_defined_libc() {
        use std::ptr::{null, null_mut};
        unsafe {
            let _ = libc::strtok(null_mut(), null()); // don't lint
            libc::strtok(null_mut(), null()); // don't lint
            libc::stderr(0); // don't lint
        }
    }
}

mod free_fns {
    use super::{time_t, tm, null_mut};

    fn localtime(_t: *const time_t) -> *mut tm {
        null_mut()
    }
    
    fn strtok() {}
    
    fn test_locatime() {
        localtime(1 as *const _); // don't lint
    }
    
    fn test_strtok() {
        strtok(); // don't lint
    }

}

fn main() {}
