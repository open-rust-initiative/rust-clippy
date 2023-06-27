#![allow(unused)]
#![warn(clippy::falliable_memory_allocation)]

fn get_untrusted_size() -> usize {
    100
}

fn allocate(_size: usize) {

}

fn main() {
    let size = get_untrusted_size();
    allocate(size);
}
