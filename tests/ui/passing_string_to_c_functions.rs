#![allow(unused)]
#![warn(clippy::passing_string_to_c_functions)]
#![feature(rustc_private)]
extern crate libc;

use std::ffi::{c_char, CString};
use libc::strlen;

extern "C" {
    fn greet(name: *const c_char);
}

fn some_string() -> String {
    "hello".to_string()
}

fn greet_safe(name: *const c_char) {}

fn main() {
    let name = String::from("Rust");
    let cstr_name = CString::new("Rust").unwrap();
    unsafe {
        greet(name.as_ptr() as *const _); // lint
        greet(cstr_name.as_ptr() as *const _); // don't lint
        greet(some_string().as_ptr() as *const _); // FN, we currently don't lint calls
        strlen(name.as_ptr() as *const _); // lint
        let c_str_ptr = cstr_name.as_ptr() as *const c_char;
        strlen(c_str_ptr); // don't lint
    }

    greet_safe(name.as_ptr() as *const _); // don't lint
}
