#![allow(unused)]
#![warn(clippy::passing_string_to_c_functions)]
use std::os::raw::c_char;

extern "C" {
    fn greet(name: *const c_char);
}

fn main() {
    let name = String::from("Rust");
    unsafe { greet(name.as_ptr() as *const _) }
}
