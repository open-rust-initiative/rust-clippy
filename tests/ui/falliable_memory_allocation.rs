#![allow(unused)]
#![warn(clippy::fallible_memory_allocation)]
#![feature(rustc_private)]
extern crate libc;

use libc::{c_void, size_t};

extern "C" {
    fn malloc(size: size_t) -> *mut c_void;
}

fn get_untrusted_size() -> usize {
    100
}

fn does_nothing_to_size(s: usize) -> usize {
    s
}

fn ptr_is_null(ptr: *mut c_void) -> bool {
    ptr.is_null()
}

unsafe fn foo1() {
    let size = get_untrusted_size();
    let p = malloc(size); // lint
}

unsafe fn foo2() {
    let size = get_untrusted_size();
    assert!(size < 100);
    let p = malloc(size); // don't lint
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    assert!(100 > size);
    let p = malloc(size); // don't lint
    assert!(!ptr_is_null(p));
}

unsafe fn size_checker() {
    fn clamp_size(s: usize) -> usize {
        s.clamp(1, 200)
    }
    fn check_size(s: usize) {
        assert!(s < 200, "size too large");
    }
    fn verified_size() -> usize {
        100
    }

    let p = malloc(1000); // don't lint
    assert!(!ptr_is_null(p));

    let size = 100;
    let p = malloc(size); // don't lint
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    let p = malloc(size.clamp(1, 200)); // don't lint
    assert!(!ptr_is_null(p));

    let size = size.clamp(1, 200);
    let p = malloc(size); // don't lint
    assert!(!ptr_is_null(p));

    let size = verified_size();
    let p = malloc(size); // don't lint
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    let p = malloc(clamp_size(size)); // don't lint
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    let good_size = clamp_size(size);
    let p = malloc(good_size); // don't lint
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    check_size(size);
    check_size(size);
    let p = malloc(size); // don't lint
    assert!(!ptr_is_null(p));

    let size = get_untrusted_size();
    let bad_size = does_nothing_to_size(size);
    let p = malloc(bad_size); // lint, currently FN
    assert!(!ptr_is_null(p));
}

unsafe fn null_ptr_checker() {
    let p = malloc(100); // lint, FN

    let size = 100;
    let p = malloc(size); // lint, FN

    let p = malloc(100); // don't lint
    assert!(!p.is_null());

    let p = malloc(100); // don't lint
    if p.is_null() {}
}

fn safe() {
    let arr: Vec<u8> = Vec::with_capacity(100); // don't lint

    let size = get_untrusted_size();
    let arr: Vec<u8> = Vec::with_capacity(size.clamp(1, 200)); // don't lint

    let size = get_untrusted_size();
    let arr: Vec<u8> = Vec::with_capacity(size); // lint

    let size = get_untrusted_size();
    let bad_size = does_nothing_to_size(size);
    let arr: Vec<u8> = Vec::with_capacity(bad_size); // lint
}

fn main() {}
