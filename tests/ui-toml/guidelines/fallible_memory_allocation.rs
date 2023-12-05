#![allow(unused)]
#![warn(clippy::fallible_memory_allocation)]

use std::collections::HashSet;
use std::ffi::c_void;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn alloc_mem(size: usize) -> *mut c_void;
    fn my_malloc(size: usize) -> *mut c_void;
}

fn get_untrusted_size() -> usize {
    100
}

fn ptr_is_null(ptr: *mut c_void) -> bool {
    ptr.is_null()
}

fn size_is_ok(s: usize) -> bool {
    s < 200
}

fn check_size(s: usize) {
    assert!(s < 200, "size too large");
}

fn exit_on_bad_size(s: usize) {
    if s > 200 {
        std::process::exit(1);
    }
}

unsafe fn foo1() {
    let size = get_untrusted_size();
    let p = malloc(size); // lint
    let p = my_malloc(size); // lint
}

unsafe fn size_checker() {
    let size = get_untrusted_size();
    let p = my_malloc(size); // lint, FN
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    check_size(size);
    let p = my_malloc(size); // lint, `check_size` not configured as a checker
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    if size_is_ok(size) {
        std::process::exit(1);
    }
    let p = my_malloc(size); // don't lint
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    assert!(size < 200, "size too large");
    let p = my_malloc(size); // don't lint
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    check_size(size);
    let p = my_malloc(size); // don't lint
    assert!(!ptr_is_null(p));
}

unsafe fn null_ptr_checker() {
    let size = get_untrusted_size();
    assert!(size < 200, "size too large");
    let p = alloc_mem(size); // lint
}

fn safe() {
    let size = get_untrusted_size();
    let arr: Vec<u8> = Vec::with_capacity(size); // lint

    let size = get_untrusted_size();
    let arr: HashSet<u8> = HashSet::with_capacity(size); // lint
}

fn main() {}
