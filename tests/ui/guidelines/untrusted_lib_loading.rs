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

fn loaded_with_default_io_functions() {
    use libloading::Library;

    let mut f = std::fs::File::open("tests/nagisa64.txt").unwrap();
    let mut name_a = String::new();
    f.read_to_string(&mut name_a).unwrap();

    let name_b = std::fs::read_to_string("foo").unwrap();
    let name_c: *mut c_char = std::ptr::null_mut();

    unsafe {
        scanf("%s".as_ptr() as *const c_char, name_c);

        let _a = libloading::Library::new(&name_a); // lint
        let _b = Library::new(name_b); // lint
        // FN: does not handle calls as argument properly, but do we want that?
        let _c = libloading::Library::new(CString::from_raw(name_c).to_string_lossy().to_string()); // lint
        let _d = Library::new("some_dylib"); // don't lint
        let _e = dlopen(name_a.as_ptr() as *const c_char, 1); // lint
    }
}

fn irrelevant_io() {
    use libloading::Library;
    use std::fs::File;

    let dll = "safe.dll";

    // Irrelevant file io
    let mut f = File::open("irrelevant_file").unwrap();
    let mut a = String::new();
    f.read_to_string(&mut a).unwrap();

    unsafe {
        let _c = Library::new(dll); // don't lint
    }
}

mod custom_scanf {
    use libloading::Library;

    fn scanf(_buf: &mut str) {}

    fn load_with_custom_scanf() {
        let mut a = String::new();
        scanf(&mut a);
        unsafe {
            let _a = Library::new(&a); // don't lint
        }
    }
}

fn main() {}
