#![allow(unused)]
#![allow(improper_ctypes, improper_ctypes_definitions)]
#![warn(clippy::extern_without_repr)]
#![feature(rustc_private)]
#![feature(repr_simd, simd_ffi)]
extern crate libc;

use std::ffi::c_int;

#[repr(C)]
struct ReprC {
    a: i32,
    b: i64,
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
struct Packed {
    a: libc::c_char,
    b: libc::c_int,
    c: libc::c_longlong,
}

#[repr(align(2))]
struct Aligned {
    inner: c_int,
}

#[repr(simd)]
struct ReprSimd {
    inner: c_int,
}

#[repr(transparent)]
struct Transparent {
    inner: c_int,
}

struct NakedStruct {
    a: libc::c_char,
}

pub enum NakedEnum {
    A,
    B,
}

extern "C" fn c_abi_fn1(arg: c_int) {}
extern "C" fn c_abi_fn2(arg_one: c_int, arg_two: ReprC) {}
extern "C" fn c_abi_fn3(arg_one: c_int, arg_two: *const Packed) {}
extern "C" fn c_abi_fn4(arg: Aligned) {}
extern "C" fn c_abi_fn5(arg: ReprSimd) {}
extern "C" fn c_abi_fn6(arg: Transparent) {}
extern "C" fn bad1(arg_one: c_int, arg_two: *const NakedStruct) {}
extern "C" fn bad2(arg: NakedStruct) {}
extern "C" fn bad3(arg: NakedEnum) {}
extern "C" fn bad4(arg: NakedEnum, arg2: NakedStruct) {}

extern "C" {
    fn c_abi_in_block1(arg_one: c_int);
    fn c_abi_in_block2(arg_one: c_int, arg_two: ReprC);
    fn c_abi_in_block3(arg_one: c_int, arg_two: Packed);
    fn c_abi_in_block4(arg: Aligned);
    fn c_abi_in_block5(arg: ReprSimd);
    fn bad_in_block1(arg_one: c_int, arg_two: NakedStruct);
    fn bad_in_block2(arg: *mut NakedEnum);
    fn bad_in_block3(arg: *mut NakedEnum, arg2: NakedStruct, arg3: *const ReprSimd);
}

fn main() {}
