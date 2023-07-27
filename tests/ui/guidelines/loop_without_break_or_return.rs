#![allow(unused, clippy::never_loop)]
#![warn(clippy::loop_without_break_or_return)]

fn test_01() {
    loop {
        println!("Hello, Rust!");
    } // lint

    loop {
        break;
    } // don't lint

    'outer: loop {
        break 'outer;
    } // don't lint

    'outer: loop {
        break;
    } // don't lint
}

fn test_02() {
    loop {
        if 2 < 3 {
            break;
        }
    } // don't lint
}

fn test_03() {
    'outer1: loop {
        for x in 0..5 {
            if x == 3 {
                break 'outer1;
            }
        }
    } // don't lint

    'outer2: loop {
        for x in 0..5 {
            if x == 3 {
                break;
            }
        } // lint
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

fn test_05() {
    fn immediate_ret() {
        loop {
            return;
        }
    }

    fn ret_in_inner() {
        'outer: loop {
            // don't lint
            'inner: loop {
                return;
            }
        }
    }

    fn cond_ret_in_inner(flag: bool) {
        'outer: loop {
            // don't lint
            'inner: loop {
                if flag {
                    return;
                }
            }
        }
    }

    fn match_ret() {
        let a = Some(0_u8);
        loop {
            match a {
                Some(v) => (),
                None => return,
            }
        }
    }
}

macro_rules! set_or_ret {
    ($opt:expr, $a:expr) => {{
        match $opt {
            Some(val) => $a = val,
            None => return,
        }
    }};
}

fn ret_in_macro(opt: Option<u8>) {
    let opt: Option<u8> = Some(1);
    let mut a: u8 = 0;
    loop {
        set_or_ret!(opt, a);
    } // don't lint
}

fn match_pat() {
    let result: Result<u8, ()> = Ok(1);
    loop {
        let val = match result {
            Ok(1) => 1 + 1,
            Ok(v) => v / 2,
            Err(_) => return,
        };
    } // don't lint

    loop {
        let Ok(val) = result else { return };
    } // don't lint

    loop {
        let Ok(val) = result.map(|v| 10) else {
            break
        }; // don't lint
    }
}

fn exhaustive_loop() {
    for i in 0..5 {
        println!("{i}");
    } // don't lint

    let mut x = 0;
    while x < 5 {
        println!("x");
        x += 1;
    } // don't lint
}

fn infinite_inner() {
    loop {
        loop {
            println!("x");
        } // lint
        break;
    }
}

fn main() {}
