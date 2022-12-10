use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, thread_rng};

fn truncate_master(slice: &str, size: u8) -> &str {
    if slice.is_char_boundary(size.into()) {
        unsafe { slice.get_unchecked(..size.into()) }
    } else if (size as usize) < slice.len() {
        let mut index = size.saturating_sub(1) as usize;
        while !slice.is_char_boundary(index) {
            index = index.saturating_sub(1);
        }
        unsafe { slice.get_unchecked(..index) }
    } else {
        slice
    }
}

fn truncate_naive(slice: &str, size: u8) -> &str {
    let mut index = size as usize;
    while !slice.is_char_boundary(index) {
        index = index.saturating_sub(1);
    }
    unsafe { slice.get_unchecked(..index) }
}

fn truncate_unrolled(slice: &str, mut size: u8) -> &str {
    if slice.is_char_boundary(size as usize) {
        return unsafe { slice.get_unchecked(..size as usize) };
    } else if size as usize >= slice.len() {
        return slice;
    }
    size -= 1;
    if slice.is_char_boundary(size as usize) {
        return unsafe { slice.get_unchecked(..size as usize) };
    }
    size -= 1;
    if slice.is_char_boundary(size as usize) {
        return unsafe { slice.get_unchecked(..size as usize) };
    }
    size -= 1;
    unsafe { slice.get_unchecked(..size as usize) }
}

/// truncates about 20% faster but could segfault if it goes to an unmapped page)
fn truncate_bits_unsafe(slice: &str, size: u8) -> &str {
    if slice.is_char_boundary(size as usize) {
        return unsafe { slice.get_unchecked(..size as usize) };
    } else if size as usize >= slice.len() {
        return slice;
    }
    unsafe {
        let size = size.saturating_sub(3);
        let ptr = (slice.as_bytes().as_ptr() as *const u32).add(size as usize);
        let data = *ptr; // could segfault if it goes over to an unmapped page
        let masked = data & 0xC0C0C0C0; // mask off only the two leftmost bis of each byte
        let zeroes = masked ^ 0x80808080; // the right values become zeroes, so we can count where the first 1 is
        let offset = zeroes.leading_zeros() / 8; // magic that gets the right
        slice.get_unchecked(..size as usize + offset as usize)
    }
}


#[inline]
fn load_u32(slice: &[u8]) -> u32 {
    match slice {
        [a, b, c, d, ..] => u32::from_le_bytes([*a, *b, *c, *d]),
        [a, b, c] => u32::from_le_bytes([*a, *b, *c, 0]),
        [a, b] => u32::from_le_bytes([*a, *b, 0, 0]),
        [a] => u32::from_le_bytes([*a, 0, 0, 0]),
        [] => 0
    }
}

fn truncate_bits(slice: &str, size: u8) -> &str {
    if slice.is_char_boundary(size as usize) {
        return unsafe { slice.get_unchecked(..size as usize) };
    } else if size as usize >= slice.len() {
        return slice;
    }
    unsafe {
        let size = size.saturating_sub(3);
        let data = load_u32(slice.as_bytes().get_unchecked(size as usize..)); // could segfault if it goes over a page limit when slice.len < 4
        let masked = data & 0xC0C0C0C0; // mask off only the two leftmost bis of each byte
        let zeroes = masked ^ 0x80808080; // the right values become zeroes, so we can count where the first 1 is
        let offset = zeroes.leading_zeros() / 8; // magic that gets the right
        slice.get_unchecked(..size as usize + offset as usize)
    }
}

fn truncate_benchmark_1byte(c: &mut Criterion) {
    let string = "abdefghijaaaaa".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&str, u8) -> &str); 5] = [
        ("truncate_master 1byte", truncate_master),
        ("truncate_naive 1byte", truncate_naive),
        ("truncate_unrolled 1byte", truncate_unrolled),
        ("truncate_bits_unsafe 1byte", truncate_bits_unsafe),
        ("truncate_bits 1byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(&string, 7);
            })
        });
    }
}


fn truncate_benchmark_2byte(c: &mut Criterion) {
    let string = "ÅÅÅÅÅÅÅÅ".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&str, u8) -> &str); 5] = [
        ("truncate_master 2byte", truncate_master),
        ("truncate_naive 2byte", truncate_naive),
        ("truncate_unrolled 2byte", truncate_unrolled),
        ("truncate_bits_unsafe 2byte", truncate_bits_unsafe),
        ("truncate_bits 2byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(&string, 7);
            })
        });
    }
}

fn truncate_benchmark_3byte(c: &mut Criterion) {
    let string = "⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠌⠍⠎⠏".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&str, u8) -> &str); 5] = [
        ("truncate_master 3byte", truncate_master),
        ("truncate_naive 3byte", truncate_naive),
        ("truncate_unrolled 3byte", truncate_unrolled),
        ("truncate_bits_unsafe 3byte", truncate_bits_unsafe),
        ("truncate_bits 3byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(&string, 7);
            })
        });
    }
}

fn truncate_benchmark_4byte(c: &mut Criterion) {
    let string = "👍👍👍👍👍👍👍".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&str, u8) -> &str); 5] = [
        ("truncate_master 4byte", truncate_master),
        ("truncate_naive 4byte", truncate_naive),
        ("truncate_unrolled 4byte", truncate_unrolled),
        ("truncate_bits_unsafe 4byte", truncate_bits_unsafe),
        ("truncate_bits 4byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(&string, 7);
            })
        });
    }
}

criterion_group!(
    truncate,
    truncate_benchmark_1byte,
    truncate_benchmark_2byte,
    truncate_benchmark_3byte,
    truncate_benchmark_4byte,
);
criterion_main!(
    truncate,
);
