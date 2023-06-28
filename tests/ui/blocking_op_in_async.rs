#![feature(async_fn_in_trait)]
#![feature(async_closure)]
#![warn(clippy::blocking_op_in_async)]
#![allow(clippy::let_underscore_future)]
#![allow(incomplete_features)]

use std::fs::read;
use std::thread::sleep;
use std::time::Duration;

pub async fn async_std_sleep() {
    sleep(Duration::from_secs(1));
}

mod totally_thread_safe {
    use std::time::Duration;

    pub async fn sleep(_dur: Duration) {}
}

mod custom_tests {
    use super::totally_thread_safe::sleep;
    use std::time::Duration;

    pub async fn conflicted_name() {
        sleep(Duration::from_secs(1)); // don't lint this
    }
}

trait AsyncTrait {
    async fn foo(&self);
    async fn bar(&mut self);
}

struct SomeType(u8);

impl AsyncTrait for SomeType {
    async fn foo(&self) {
        sleep(Duration::from_secs(self.0 as _));
    }
    // don't lint
    async fn bar(&mut self) {
        self.0 = 1;
    }
}

fn do_something() {}

#[rustfmt::skip]
fn closures() {
    let _ = async || {
        sleep(Duration::from_secs(1))
    };
    let async_closure = async move |_a: i32| {
        let _ = 1;
        do_something();
        sleep(Duration::from_secs(1));
    };
    // don't lint, not async
    let non_async_closure = |_a: i32| {
        sleep(Duration::from_secs(1));
        do_something();
    };
}

fn main() {}
