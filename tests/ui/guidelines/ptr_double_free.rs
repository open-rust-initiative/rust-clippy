#![allow(unused)]
#![warn(clippy::ptr_double_free)]
#![feature(rustc_private)]
extern crate libc;

use core::ffi::c_void;
use core::ptr::{null, null_mut};

#[allow(non_camel_case_types)]
type size_t = usize;

extern "C" {
    fn malloc(size: size_t) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

fn do_something() {}

macro_rules! free {
    ($ptr:expr) => {
        libc::free($ptr);
    };
}

unsafe fn double_free() {
    let ptr: *mut c_void = malloc(1);

    free(ptr);
    free(ptr); // lint

    let libc_ptr = libc::malloc(1);
    libc::free(libc_ptr);
    libc::free(libc_ptr); // lint

    let libc_ptr_1 = libc::malloc(1);
    free(libc_ptr_1);
    do_something();
    libc::free(libc_ptr_1); // lint

    let libc_ptr_2 = libc::malloc(1);
    free!(libc_ptr_2);
    do_something();
    free(libc_ptr_2); // lint

    let ptr_1: *mut c_void = malloc(1);
    free!(ptr_1);
    do_something();
    free!(ptr_1); // FN: unknown reason
}

unsafe fn free_with_null_check() {
    let mut ptr: *mut c_void = malloc(1);
    free(ptr);
    ptr = null_mut();
    if !ptr.is_null() {
        free(ptr); // don't lint
    }

    let mut ptr: *mut c_void = malloc(1);
    free(ptr);
    ptr = null_mut();
    if ptr.is_null() {
        free(ptr); // don't lint: freeing a null pointer, should be a separated lint
    }

    let mut ptr: *mut c_void = malloc(1);
    if !ptr.is_null() {
        free(ptr);
        ptr = null_mut();
    }
    if !ptr.is_null() {
        free(ptr); // don't lint
        ptr = null_mut();
    }
}

fn main() {}
