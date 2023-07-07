#![allow(unused, clippy::never_loop)]
#![warn(clippy::loop_without_break_or_return)]

fn test_01() {
    loop {
        println!("Hello, Rust!");
    }

    loop {
        break;
    }

    'outer: loop {
        break 'outer;
    }

    'outer: loop {
        break;
    }
}

fn test_02() {
    loop {
        if 2 < 3 {
            break;
        }
    }
}

fn test_03() {
    'outer1: loop {
        for x in 0..5 {
            if x == 3 {
                break 'outer1;
            }
        }
    }

    'outer2: loop {
        for x in 0..5 {
            if x == 3 {
                break;
            }
        }
    }

    'outer3: loop {
        for x in 0..5 {
            if x == 3 {
                println!("Hello, Rust!");
            } else {
                break 'outer3;
            }
        }
    }
}

fn test_04() {
    'outer1: loop {
        loop {
            println!("Hello, Rust!");
        }
        break;
    }

    'outer2: loop {
        loop {
            break;
        }
    }

    'outer3: loop {
        loop {
            break 'outer3;
        }
    }

    'outer4: loop {
        'inner: loop {
            loop {
                break 'inner;
            }
        }
    }

    'outer5: loop {
        loop {
            'inner: loop {
                loop {
                    loop {
                        break 'inner;
                    }
                    break 'outer5;
                }
            }
        }
    }
}

fn main() {
    // test code goes here
}
