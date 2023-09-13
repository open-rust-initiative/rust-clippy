#![allow(unused)]
#![warn(clippy::dangling_ptr_dereference)]

fn my_free(_ptr: *mut i8) {}

fn main() {
    unsafe {
        let ptr: *mut i8 = 1 as *mut _;
        my_free(ptr);
        let _ = *ptr; // lint

        let ptr_1: *mut i8 = 1 as *mut _;
        my_free(ptr_1);
        println!("{:?}", *ptr_1); // lint

        let ptr_2: *mut i8 = 1 as *mut _;
        my_free(ptr_2);
        if !ptr_2.is_null() {
            println!("{:?}", *ptr_2); // don't lint
        }
    }
}
