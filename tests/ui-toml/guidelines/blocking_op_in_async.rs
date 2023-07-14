#![feature(async_closure)]
#![warn(clippy::blocking_op_in_async)]
#![allow(clippy::let_underscore_future)]
#![allow(incomplete_features)]

use std::fs::{self, read};
use std::thread::sleep;
use std::time::Duration;

fn do_something() {}

pub async fn async_std_read() {
    let _ = read("non_exist.txt");
}

pub async fn async_std_fn_paths() -> std::io::Result<()> {
    std::fs::remove_dir("non_exist_dir")?;
    std::fs::write("non_exist.txt", b"Lorem ipsum")?;
    fs::copy("non_exist.txt", "non_exist_1.txt")?;
    let _ = fs::canonicalize("./non/exist/dir/");
    Ok(())
}

// don't lint anything in this function
pub fn non_async_std_read() {
    let _ = read("non_exist.txt");
    let _ = std::fs::write("non_exist.txt", b"Lorem ipsum");
}

mod totally_thread_safe {
    use std::path::Path;
    use std::time::Duration;

    pub async fn sleep(_dur: Duration) {}
    pub fn read<P: AsRef<Path>>(_path: P) {}
    pub async fn write<P: AsRef<Path>, C: AsRef<[u8]>>(_path: P, _content: C) -> Result<(), ()> {
        Ok(())
    }
}

mod custom_tests {
    use super::totally_thread_safe::*;
    use std::fs::create_dir;
    use std::time::Duration;

    pub async fn conflicted_name_fns() {
        sleep(Duration::from_secs(1)); // don't lint this
        read("non_exist.txt"); // don't lint this
        write("non_exist.txt", b"Lorem ipsum"); // don't lint this
        create_dir("foo");
    }
}

fn closures() {
    let _ = async || read("non_exist.txt");
    let _ = async || {
        read("non_exist.txt");
        std::fs::create_dir("foo");
    };
    use totally_thread_safe::write;
    let async_closure = async move |_a: i32| {
        write("non_exist.txt", b"Lorem ipsum"); // don't lint this
        fs::create_dir("foo");
        sleep(Duration::from_secs(1));
        let _res = fs::copy("a", "b");
    };
    // don't lint
    let non_async_closure = |_a: i32| {
        fs::create_dir("foo");
    };
}

#[allow(clippy::single_match)]
pub async fn if_and_matches() -> Result<(), ()> {
    if read("foo").is_ok() {
        do_something();
    }
    if let Ok(f) = read("foo") {
        do_something();
    }
    match read("foo") {
        Ok(_a) => do_something(),
        Err(_) => (),
    }
    match read("foo") {
        Ok(_a) => {
            fs::copy("foo", "foo_1").unwrap();
        },
        Err(_) => {
            fs::write("foo", b"bar").unwrap();
        },
    }
    // False Negative ALERT!!!
    // currently ignoring normal function with blocking operations,
    // as it'll get very obnoxious to deal with.
    fn just_read() -> std::io::Result<Vec<u8>> {
        read("foo")?;
        let _ = read("bar")?;
        read("baz")
    }

    let cls = || {
        fs::copy("foo", "bar").unwrap();
    };
    read("non_exist.txt").map_err(|_| ())?;
    read("foo").map(|_| ()).map_err(|_| ())?;
    just_read().map_err(|_| ())?;
    cls();
    Ok(())
}

fn main() {}
