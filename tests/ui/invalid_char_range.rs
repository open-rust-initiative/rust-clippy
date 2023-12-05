#![warn(clippy::invalid_char_range)]

const VALID: u32 = 0xD000;
const INVALID_BETWEEN: u32 = 0xDDDD;
const INVALID_LARGE_SIGNED: i64 = 0xFFFFFF;

pub fn valid() {
    let _ = char::from(97u8);
    let _ = char::from_u32(0x10FFFF);
    let _ = char::from_u32(VALID);
}

pub fn invalid() {
    let _ = char::from_u32(0xD800);
    //~^ ERROR: converting to char with out-of-range integer
    //~| NOTE: `-D clippy::invalid-char-range` implied by `-D warnings`
    let _ = unsafe { char::from_u32_unchecked(0xD800) };
    //~^ ERROR: converting to char with out-of-range integer
    let _ = char::from_u32(INVALID_BETWEEN);
    //~^ ERROR: converting to char with out-of-range integer
    let _ = char::from_u32(INVALID_BETWEEN + 1);
    //~^ ERROR: converting to char with out-of-range integer
    let _ = unsafe { char::from_u32_unchecked(INVALID_BETWEEN + 1) };
    //~^ ERROR: converting to char with out-of-range integer
    let _ = char::from_u32(INVALID_LARGE_SIGNED as u32);
    //~^ ERROR: converting to char with out-of-range integer
    let _ = char::from_u32(INVALID_LARGE_SIGNED as usize as u32);
    //~^ ERROR: converting to char with out-of-range integer
}

mod my_char {
    pub fn from_u32(u: u32) -> char {
        'a'
    }
}

pub fn skip() {
    let _ = my_char::from_u32(INVALID_BETWEEN);
}

fn get_num() -> u32 {
    100
}
// Don't lint uncertain results
pub fn uncertain(n: u32) {
    const INVALID_BETWEEN: u32 = 0xDDDD;
    let _ = char::from_u32(INVALID_BETWEEN.saturating_sub(1));

    let _ = char::from_u32(n);
    let _ = char::from_u32(10 + n);
    let _ = char::from_u32(10_u32.saturating_add(n));

    let _ = char::from_u32(get_num());
    let clo = || 0;
    let _ = char::from_u32(clo());
}

fn main() {}
