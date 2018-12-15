extern crate arraystring;

use arraystring::{LimitedString, ArrayString};

fn main() {
    const COUNT: usize = 10_000_000;
    for _ in 0..COUNT {
        unsafe { LimitedString::from_str_unchecked("ashdash") };
    }
}
