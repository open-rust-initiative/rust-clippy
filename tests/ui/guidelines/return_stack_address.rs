#![allow(unused)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::needless_late_init)]
#![warn(clippy::return_stack_address)]

#[derive(Clone)]
struct NonCopiable(Vec<u8>);

impl NonCopiable {
    fn as_ptr(&self) -> *const u8 {
        1_u8 as *const u8
    }
}

#[derive(Clone, Copy)]
struct Copiable(u8);

impl Copiable {
    fn as_ptr(&self) -> *const u8 {
        self.0 as *const _
    }
}

trait ToPtr {
    fn to_ptr(self) -> *const u8;
}

impl ToPtr for u8 {
    fn to_ptr(self) -> *const u8 {
        1 as *const u8
    }
}

fn fn_ret() {
    fn implicit_int() -> *const i32 {
        let val: i32 = 100;
        &val as *const _ // lint
    }

    fn implicit_int_mut() -> *mut i32 {
        let mut val: i32 = 100;
        &mut val as *mut _ // lint
    }

    fn explicit_int() -> *const i32 {
        let val: i32 = 100;
        return &val as *const _; // lint
    }

    fn both_int() -> *const i32 {
        let val: i32 = 100;
        let val_2: i32 = 200;
        if val > 0 {
            return &val_2 as *const _; // lint
        }
        &val as *const _ // lint
    }

    fn ret_str() -> *const u8 {
        let val = "123";
        val.as_ptr() // lint
    }

    fn ret_str_as_ptr() -> *const i8 {
        let val = "123";
        val.as_ptr() as *const _ // lint
    }

    fn chained_method_call() -> *const u8 {
        let val = "123";
        val.to_string().as_ptr() // lint
    }

    fn chained_cast() -> *const i32 {
        let val: u8 = 1;
        &val as *const u8 as *const i8 as *const i16 as *const i32 // lint
    }

    fn complicated() -> *const core::ffi::c_char {
        let val = "123";
        val.to_string().clone().as_ptr() as *const i8 as *const i32 as *const _ // lint
    }

    fn non_simple_ty() -> *const u8 {
        let val = String::from("123");
        val.as_ptr() // don't lint
    }

    fn declare_then_assign() -> *const i32 {
        let val;
        val = 12;
        &val as *const _ // lint
    }

    fn ret_non_copiable() -> *const u8 {
        let val: NonCopiable = NonCopiable(vec![1, 2, 3]);
        val.as_ptr() // don't lint
    }

    fn ret_copiable() -> *const u8 {
        let val: Copiable = Copiable(123);
        val.as_ptr() // Maybe FN?
    }

    fn ret_other_call() -> *const u8 {
        let val: u8 = 20;
        val.to_ptr() // don't lint
    }
}

fn block_ret() {
    let x = {
        let val: i32 = 100;
        &val as *const i32 // lint
    };

    let x = match Some(1) {
        Some(n) => {
            let val: i32 = n + 1;
            &val as *const i32 // lint
        },
        None => {
            2 as *const i32 // don't lint
        },
    };
}

fn main() {}
