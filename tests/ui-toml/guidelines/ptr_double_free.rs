#![allow(unused)]
#![warn(clippy::ptr_double_free)]

fn my_free(_ptr: *mut i8) {}

fn main() {
    let ptr: *mut i8 = 1 as *mut _;

    my_free(ptr);
    my_free(ptr); // lint

    let ptr1: *mut i8 = 1 as *mut _;
    let ptr2: *mut i8 = 1 as *mut _;
    let ptr3: *mut i8 = 1 as *mut _;
    my_free(ptr1);
    my_free(ptr2);
    my_free(ptr1); // lint
    my_free(ptr3);
}
