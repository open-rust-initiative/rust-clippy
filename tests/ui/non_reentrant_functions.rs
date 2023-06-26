#![warn(clippy::non_reentrant_functions)]
#![allow(unused)]
#![feature(rustc_private)]
extern crate libc;

use std::ffi::{CStr, CString};

fn test_libc_localtime() {
    // test code goes here
    unsafe {
        let _tm = libc::localtime(&0i64 as *const libc::time_t);
    }
}

fn test_libc_strtok() {
    let string = CString::new("welcome-to-rust").unwrap();
    let string = string.as_ptr() as *mut libc::c_char;
    let delim = CString::new(" - ").unwrap();
    let delim = delim.as_ptr();

    let mut token = unsafe { libc::strtok(string, delim) };
    while !token.is_null() {
        println!("{:?}", unsafe { CStr::from_ptr(token) });
        token = unsafe { libc::strtok(std::ptr::null_mut(), delim) };
    }
}

fn test_locatime() {
    fn localtime() {}
    localtime();
}

fn test_strtok() {
    fn strtok() {}
    strtok();
}

fn main() {}
