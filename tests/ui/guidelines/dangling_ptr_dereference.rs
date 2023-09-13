#![allow(unused)]
#![warn(clippy::dangling_ptr_dereference)]
#![feature(rustc_private)]
extern crate libc;

use core::ffi::c_void;

#[allow(non_camel_case_types)]
type size_t = usize;

extern "C" {
    fn malloc(size: size_t) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

unsafe fn free_and_deref() {
    let ptr: *mut c_void = malloc(1);

    free(ptr);
    let _ = *ptr;

    let ptr_libc: *mut c_void = libc::malloc(1);
    libc::free(ptr_libc);
    println!("{:?}", *ptr_libc);

    let ptr_1: *mut c_void = malloc(1);
    free(ptr_1);
    if !ptr_1.is_null() {
        println!("{:?}", *ptr_1);
    }

    let ptr_2: *mut c_void = malloc(1);
    if !ptr_2.is_null() {
        free(ptr_2); // FN, parent block of this free is not the whole function body
    }
    if !ptr_2.is_null() {
        println!("{:?}", *ptr_2);
    }
}

// Don't lint anything inside
unsafe fn free_but_replaced() {
    let mut ptr: *mut c_void = malloc(1);

    free(ptr);
    ptr = malloc(1);
    let _ = *ptr;

    let shadow: *mut c_void = malloc(1);
    free(shadow);
    let shadow: *mut c_void = libc::malloc(1);
    let _ = *shadow;
}

fn main() {}
