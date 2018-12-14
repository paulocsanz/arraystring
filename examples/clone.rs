extern crate limited_string;

use limited_string::{LimitedString, StringHandler};

fn main() {
    const COUNT: usize = 10_000_000;
    for _ in 0..COUNT {
        unsafe { LimitedString::from_str_unchecked("ashdash") };
    }
}
