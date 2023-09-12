#![allow(unused)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::needless_late_init)]
#![warn(clippy::null_ptr_dereference)]

use std::ptr::{null, null_mut};

struct Pointer {
    inner: i8,
}

impl Pointer {
    fn as_ptr(&self) -> *const i8 {
        (&self.inner) as *const _
    }
}

fn do_something(_p: i8) {}

fn init_null_ptrs() {
    let a: *mut Pointer = null_mut();
    unsafe {
        do_something((*a).inner);
    }
}

fn assign_null_ptrs() {
    let a: *mut Pointer;
    a = null_mut();
    unsafe {
        do_something((*a).inner);
    }

    let mut b: *mut Pointer;
    b = 0 as *mut Pointer;
    unsafe {
        do_something((*b).inner);
    }

    let mut reassign_later: *const Pointer = 1 as *const _;
    reassign_later = null();
    let _ = unsafe { (*reassign_later).inner };
    reassign_later = 0 as *const _;
    unsafe { if (*reassign_later).inner > 0 {} }
}

fn deref_in_macros() {
    let a: *mut Pointer = 0 as *mut _;
    unsafe {
        println!("{}", (*a).inner);
    }

    let foo_ptr: *mut Pointer = std::ptr::null_mut();
    unsafe {
        println!("{:?}", (*foo_ptr).inner);
    }
}

unsafe fn misc_deref() {
    let mut still_null: *mut i8 = null_mut();
    still_null = 0 as *mut i8;
    let _ = *still_null;

    let as_train: *mut i8;
    as_train = 0 as *mut i32 as *mut u16 as *mut i8;
    let _ = *as_train;
}

// Don't lint anything in this function
// There might be seg-fault in actual code, but it's ok, because we are not testing that.
unsafe fn deref_non_null() {
    let mut as_p: *mut Pointer = 0 as *mut _;
    let mut null_p: *const i8 = null();
    let mut null_mut_p: *mut i8 = null_mut();
    let mut null_pointer: *const Pointer = null();

    fn assign_pointer(mut p: *const Pointer, pointer: &Pointer) {
        p = pointer.as_ptr() as *const Pointer;
    }

    as_p = 1 as *mut _;
    println!("{:?}", (*as_p).inner);

    let good_pointer = Pointer { inner: 1 };
    null_p = good_pointer.as_ptr();
    println!("{:?}", *null_p);

    null_mut_p = 1 as *mut i32 as *mut i16 as *mut i8;
    println!("{:?}", *null_mut_p);

    assign_pointer(null_pointer, &good_pointer);
    println!("{:?}", (*null_pointer).inner);
}

// Don't anything in this function
unsafe fn checked_deref() {
    let as_p: *mut Pointer = 0 as *mut _;
    let null_p: *const i8 = null();

    if !as_p.is_null() {
        println!("{:?}", (*as_p).inner);
    }
    if !null_p.is_null() {
        do_something(*null_p);
    }
}

unsafe fn wrongly_checked_deref() {
    let null_0: *const i8 = null();
    let null_o: *const i8 = null();

    if !null_0.is_null() {
        do_something(*null_o);
    }
    if !null_o.is_null() {
        do_something(*null_0); // FIXME: False negative
    }
}

#[allow(clippy::no_effect)]
unsafe fn override_value() {
    let null_p: *mut i8 = null_mut();
    *null_p = 1_i8;
    *null_p;
}

fn main() {}
