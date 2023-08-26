#![crate_type = "proc-macro"]
#![warn(clippy::unsafe_block_in_proc_macro)]
#![allow(clippy::needless_return)]

extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;
use quote::quote;

fn do_something() {}
fn main() {}

#[proc_macro]
pub fn unsafe_print_foo(_: TokenStream) -> TokenStream {
    quote!({
        unsafe {
            println!("foo");
        }
    })
    .into()
}

#[rustfmt::skip]
#[proc_macro]
pub fn unsafe_print_bar_unformatted(_: TokenStream) -> TokenStream {
    quote!({
        unsafe
        
        
        { println!("bar"); }
    }).into()
}

#[rustfmt::skip]
#[proc_macro]
pub fn unsafe_print_bar_unformatted_1(_: TokenStream) -> TokenStream {
    quote!({
        unsafe{println!("bar");}
    }).into()
}

#[proc_macro]
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

#[proc_macro]
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

#[proc_macro]
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

#[proc_macro]
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

#[proc_macro]
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

#[proc_macro]
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

#[proc_macro]
// Don't lint
pub fn print_foo(_: TokenStream) -> TokenStream {
    quote!({
        println!("foo");
    })
    .into()
}

#[proc_macro]
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

#[proc_macro]
// Don't lint
pub fn unsafe_fn(_: TokenStream) -> TokenStream {
    quote!({
        unsafe fn uns_foo() {}
    })
    .into()
}

#[proc_macro]
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
