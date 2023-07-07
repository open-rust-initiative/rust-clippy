#![allow(unused)]
#![allow(improper_ctypes_definitions)]
#![allow(improper_ctypes)]
#![warn(clippy::extern_without_repr)]
#![feature(rustc_private)]
#![feature(core_intrinsics)]
extern crate libc;

#[repr(C)]
struct Foo1 {
    a: i32,
    b: i64,
    c: u128,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
struct Foo2 {
    a: libc::c_char,
    b: libc::c_int,
    c: libc::c_longlong,
}

struct Foo3 {
    a: libc::c_char,
    b: libc::c_int,
    c: libc::c_longlong,
}

extern "C" fn c_abi_fn1(arg_one: u32, arg_two: usize) {}
extern "C" fn c_abi_fn2(arg_one: u32, arg_two: Foo1) {}
extern "C" fn c_abi_fn3(arg_one: u32, arg_two: *const Foo2) {}
extern "C" fn c_abi_fn4(arg_one: u32, arg_two: *const Foo3) {}

extern "C" {
    fn c_abi_in_block1(arg_one: u32, arg_two: usize);
    fn c_abi_in_block2(arg_one: u32, arg_two: Foo1);
    fn c_abi_in_block3(arg_one: u32, arg_two: Foo2);
    fn c_abi_in_block4(arg_one: u32, arg_two: Foo3);
}

fn main() {
    // test code goes here
}
