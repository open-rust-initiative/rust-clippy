//@aux-build:../auxiliary/proc_macros.rs

#![feature(lint_reasons)]
#![warn(clippy::unconstrained_numeric_literal)]
#![allow(clippy::let_with_type_underscore, clippy::let_and_return)]

extern crate proc_macros;
use proc_macros::{external, inline_macros};

mod basic_expr {
    fn test() {
        let x = 22;
        //~^ ERROR: type of this numeric variable is unconstrained
        //~| NOTE: `-D clippy::unconstrained-numeric-literal` implied by `-D warnings`
        let x = 22.0;
        //~^ ERROR: type of this numeric variable is unconstrained
        let x = [1, 2, 3];
        //~^ ERROR: type of this numeric variable is unconstrained
        let x = if true { (1, 2) } else { (3, 4) };
        //~^ ERROR: type of this numeric variable is unconstrained
        let x = if true { (1.0, 2, 3.0) } else { (3.0, 4, 5.0) };
        //~^ ERROR: type of this numeric variable is unconstrained
        let x = match 1 {
            //~^ ERROR: type of this numeric variable is unconstrained
            1 => 1,
            _ => 2,
        };
        // Has type annotation but it's a wildcard.
        let x: _ = 1;
        //~^ ERROR: type of this numeric variable is unconstrained

        let x = 22_i32;
        let x: [i32; 3] = [1, 2, 3];
        let x: (i32, i32) = if true { (1, 2) } else { (3, 4) };
        let x: u64 = 1;
        const CONST_X: i8 = 1;
    }
}

mod nested_local {
    fn test() {
        let x = {
            //~^ ERROR: type of this numeric variable is unconstrained
            let y = 1;
            //~^ ERROR: type of this numeric variable is unconstrained
            1
        };

        let x: i32 = {
            let y = 1;
            //~^ ERROR: type of this numeric variable is unconstrained
            1
        };

        const CONST_X: i32 = {
            let y = 1;
            //~^ ERROR: type of this numeric variable is unconstrained
            1
        };
    }
}

mod in_macro {
    use super::*;

    #[inline_macros]
    fn internal() {
        inline!(let x = 22;);
        //~^ ERROR: type of this numeric variable is unconstrained
    }

    fn external() {
        external!(let x = 22;);
    }
}

fn check_expect_suppression() {
    #[expect(clippy::unconstrained_numeric_literal)]
    let x = 21;
}

fn main() {}
