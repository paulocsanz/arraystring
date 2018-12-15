extern crate arraystring;

use arraystring::prelude::*;

fn main() {
    const COUNT: usize = 10_000_000;
    for _ in 0..COUNT {
        unsafe { CacheString::from_str_unchecked("ashdash") };
    }
}
