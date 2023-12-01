#![allow(unused)]
#![warn(clippy::implicit_abi)]

use std::ffi::{c_char, c_void};

#[repr(C)]
pub struct External(i8);

#[rustfmt::skip]
extern {
    fn foo();
    fn bar(a: u8, b: i32) -> u64;
    pub fn baz(a: c_char) -> c_void;
    pub fn qux(a: External) -> c_void;

    pub static a: std::ffi::c_int;
    pub static b: External;
    static c: i8;
}

#[rustfmt::skip]
extern
{
    static SOME_PTR: *mut c_void;
}

#[rustfmt::skip]
extern { fn my_c_fn(a: i8) -> c_void; }

// For reference, don't lint
extern "C" {
    fn c_func();
}
// For reference, don't lint
extern "system" {
    pub fn system_call();
}

fn main() {}
