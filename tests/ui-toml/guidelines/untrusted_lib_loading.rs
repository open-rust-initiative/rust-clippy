#![allow(unused)]
#![warn(clippy::untrusted_lib_loading)]
#![feature(rustc_private)]

extern crate libloading;
use std::ffi::{c_char, c_int, c_void, CString};
use std::io::Read;

extern "C" {
    fn scanf(format: *const c_char, buf: *mut c_char) -> i32;
    fn dlopen(f: *const c_char, flag: c_int) -> *mut c_void;
}

fn untrusted_io() -> String {
    "bad".to_string()
}

fn load_dylib(_lib: &str) -> i32 {
    0
}

mod lib_loader {
    pub fn load(_lib: &str) -> i32 {
        0
    }
}

fn load_using_libloading_and_custom_io() {
    use libloading::Library;

    let name_a = untrusted_io();
    let name_b = &untrusted_io();

    unsafe {
        let _a = Library::new(name_a); // lint
        let _b = libloading::Library::new(name_b); // lint
    }
}

fn load_using_dlopen_and_custom_io() {
    let name_a = untrusted_io().as_ptr() as *const c_char;
    let name_b = untrusted_io();

    unsafe {
        let _a = dlopen(name_a, 1); // lint
        let _b = dlopen(name_b.as_ptr() as *const c_char, 1); // lint
    }
}

fn custom_loaders() {
    use lib_loader::load;

    let name_a = std::fs::read_to_string("foo").unwrap();

    let _a = load_dylib(&name_a); // lint
    let _b = load(&name_a); // lint
    let _c = lib_loader::load(&name_a); // lint
}

fn main() {}
