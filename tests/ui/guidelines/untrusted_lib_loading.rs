#![allow(unused)]
#![warn(clippy::untrusted_lib_loading)]
#![feature(rustc_private)]

extern crate libloading;
use std::io::Read;

fn main() {
    let mut f = std::fs::File::open("tests/nagisa64.txt").unwrap();
    let mut name = String::new();
    f.read_to_string(&mut name).unwrap();

    unsafe {
        let lib = libloading::Library::new(&name);
        if lib.is_err() {
            println!("can't open {}", name);
        }
    }
}
