#![warn(clippy::non_reentrant_functions)]
#![feature(rustc_private)]
extern crate libc;

#[allow(unused)]
use libc::{c_char, localtime, strtok, time_t};
use std::ffi::{CStr, CString};

fn main() {
    // test code goes here
    unsafe {
        let _tm = localtime(&0i64 as *const time_t);
    }

    let string = CString::new("welcome-to-rust").unwrap();
    let string = string.as_ptr() as *mut c_char;
    let delim = CString::new(" - ").unwrap();
    let delim = delim.as_ptr();

    let mut token = unsafe { strtok(string, delim) };
    while !token.is_null() {
        println!("{:?}", unsafe { CStr::from_ptr(token) });
        token = unsafe { strtok(std::ptr::null_mut(), delim) };
    }
}
