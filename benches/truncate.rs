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
    let size = size as usize;
    match slice.as_bytes().get(size).map(|it| *it & 0xC0 != 0x80) {
        Some(true) => unsafe { slice.get_unchecked(..size) },
        Some(false) => unsafe {
            let size = size.saturating_sub(3);
            let data = load_u32(slice.as_bytes().get_unchecked(size..)); // could segfault if it goes over a page limit when slice.len < 4
            let masked = data & 0xC0C0C0C0; // mask off only the two leftmost bis of each byte
            let zeroes = masked ^ 0x80808080; // the right values become zeroes, so we can count where the first 1 is that indicates the first non boundary
            let offset = zeroes.leading_zeros() / 8; // magic that gets the right
            slice.get_unchecked(..size + 3 - offset as usize)
        },
        None => slice,
    }
}

fn truncate_benchmark_1byte(c: &mut Criterion) {
    let string = "abdefghijaaaaa".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&str, u8) -> &str); 4] = [
        ("truncate_master 1byte", truncate_master),
        ("truncate_naive 1byte", truncate_naive),
        ("truncate_unrolled 1byte", truncate_unrolled),
        ("truncate_bits 1byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        for (name2, f2) in TESTS {
            assert_eq!(f(&string, 7), f2(&string, 7), "{} != {}", name, name2);
        }
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(&string, 7);
            })
        });
    }
}


fn truncate_benchmark_2byte(c: &mut Criterion) {
    let string = "ÅÅÅÅÅÅÅÅ".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&str, u8) -> &str); 4] = [
        ("truncate_master 2byte", truncate_master),
        ("truncate_naive 2byte", truncate_naive),
        ("truncate_unrolled 2byte", truncate_unrolled),
        ("truncate_bits 2byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        for (name2, f2) in TESTS {
            assert_eq!(f(&string, 7), f2(&string, 7), "{} != {}", name, name2);
        }
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(&string, 7);
            })
        });
    }
}

fn truncate_benchmark_3byte(c: &mut Criterion) {
    let string = "⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠌⠍⠎⠏".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&str, u8) -> &str); 4] = [
        ("truncate_master 3byte", truncate_master),
        ("truncate_naive 3byte", truncate_naive),
        ("truncate_unrolled 3byte", truncate_unrolled),
        ("truncate_bits 3byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        for (name2, f2) in TESTS {
            assert_eq!(f(&string, 7), f2(&string, 7), "{} != {}", name, name2);
        }
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(&string, 7);
            })
        });
    }
}

fn truncate_benchmark_4byte(c: &mut Criterion) {
    let string = "👍👍👍👍👍👍👍".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&str, u8) -> &str); 4] = [
        ("truncate_master 4byte", truncate_master),
        ("truncate_naive 4byte", truncate_naive),
        ("truncate_unrolled 4byte", truncate_unrolled),
        ("truncate_bits 4byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        for (name2, f2) in TESTS {
            assert_eq!(f(&string, 7), f2(&string, 7), "{} != {}", name, name2);
        }
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
