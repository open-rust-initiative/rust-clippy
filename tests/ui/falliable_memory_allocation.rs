#![allow(unused)]
#![warn(clippy::falliable_memory_allocation)]

fn get_untrusted_size() -> usize {
    100
}

fn allocate(size: usize) -> usize {
    size
}

fn is_null(ptr: usize) -> bool {
    false
}

fn foo1() {
    let size = get_untrusted_size();
    let p = allocate(size);
}

fn foo2() {
    let size = get_untrusted_size();
    assert!(size < 100);
    let p = allocate(size);
    assert!(!is_null(p));
}

fn main() {}
