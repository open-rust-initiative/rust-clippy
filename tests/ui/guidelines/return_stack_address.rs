#![allow(unused)]
#![allow(clippy::needless_return)]
#![allow(clippy::redundant_clone)]
#![warn(clippy::return_stack_address)]

fn fn_ret() {
    fn implicit_int() -> *const i32 {
        let val: i32 = 100;
        &val as *const _ // lint
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
