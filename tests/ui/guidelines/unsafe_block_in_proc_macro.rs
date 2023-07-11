#![warn(clippy::unsafe_block_in_proc_macro)]
#![allow(clippy::needless_return)]

extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;
use quote::quote;

fn do_something() {}
fn main() {}

// Ideally we can add `#[proc_macro]` attr on these following functions,
// and check only such functions instead, but I don't know how to write such test
// without putting `proc_macro = true` in Cargo.toml, maybe someone else can fix it.

pub fn unsafe_print_foo(_: TokenStream) -> TokenStream {
    quote!({
        unsafe {
            println!("foo");
        }
    })
    .into()
}

#[rustfmt::skip]
pub fn unsafe_print_bar_unformatted(_: TokenStream) -> TokenStream {
    quote!({
        unsafe
        
        
        { println!("bar"); }
    }).into()
}

#[rustfmt::skip]
pub fn unsafe_print_bar_unformatted_1(_: TokenStream) -> TokenStream {
    quote!({
        unsafe{println!("bar");}
    }).into()
}

pub fn unsafe_print_baz(_: TokenStream) -> TokenStream {
    do_something();

    quote!({
        let _ = 0_u8;
        do_something();
        unsafe {
            println!("baz");
        }

        do_something();
    })
    .into()
}

pub fn maybe_unsafe_print(_: TokenStream) -> TokenStream {
    do_something();

    if false {
        return quote!({
            unsafe {
                println!("unsafe");
            }
        })
        .into();
    }

    quote!({
        println!("safe");
    })
    .into()
}

pub fn maybe_unsafe_print_1(_: TokenStream) -> TokenStream {
    let condition = 1;
    if condition > 0 {
        return quote!({
            println!("safe");
        })
        .into();
    }

    quote!({
        unsafe {
            println!("unsafe");
        }
    })
    .into()
}

pub fn maybe_unsafe_print_2(_: TokenStream) -> TokenStream {
    let condition = 1;
    if condition == 0 {
        return quote!({}).into();
    } else if condition == 1 {
        return quote!(unsafe {}).into();
    } else if condition == 2 {
        return quote!({ println!("2") }).into();
    }
    do_something();
    quote!({}).into()
}

pub fn multiple_unsafe(_: TokenStream) -> TokenStream {
    let condition = 1;
    match condition {
        1 => quote!(unsafe {
            println!("1");
        }),
        2 => quote!(unsafe {
            println!("2");
        }),
        _ => quote!(unsafe {}),
    }
    .into()
}

pub fn unsafe_block_in_function(_: TokenStream) -> TokenStream {
    quote!({
        fn print_foo() {
            unsafe {
                println!("foo");
            }
        }
    })
    .into()
}

// Don't lint
pub fn print_foo(_: TokenStream) -> TokenStream {
    quote!({
        println!("foo");
    })
    .into()
}

// Don't lint
pub fn unsafe_trait(_: TokenStream) -> TokenStream {
    quote!({
        unsafe trait Foo {
            fn aaa();
            fn bbb();
        }
    })
    .into()
}

// Don't lint
pub fn unsafe_fn(_: TokenStream) -> TokenStream {
    quote!({
        unsafe fn uns_foo() {}
    })
    .into()
}

// Don't lint
pub fn unsafe_impl_fn(_: TokenStream) -> TokenStream {
    quote!({
        struct T;
        impl T {
            unsafe fn foo() {}
        }
    })
    .into()
}
