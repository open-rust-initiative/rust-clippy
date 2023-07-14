#![warn(clippy::mem_unsafe_functions)]
#![allow(unused)]

mod libc {
    pub fn safe() {}
    pub fn not_safe() {}
    pub fn memcpy() {}
}

extern "C" {
    fn safe();
    fn safe_1();
    fn not_safe();
    fn dont_use();
    fn del_mem();
    fn memcpy();
}

fn main() {
    unsafe {
        // Don't trigger
        safe();
        safe_1();
        libc::safe();
        libc::not_safe();
        libc::memcpy(); // because it's user defined

        // should trigger
        not_safe();
        dont_use();
        del_mem();
        memcpy();
    }
}
