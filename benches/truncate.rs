use criterion::{criterion_group, criterion_main, Criterion};
use rand::{thread_rng, Rng};

#[inline(always)]
fn truncate_naive(slice: &[u8], mut size: usize) -> &[u8] {
    if size >= slice.len() {
        return slice;
    }
    while size > 0 {
        unsafe {
            if is_char_boundary_at(slice, size) {
                return slice.get_unchecked(..size);
            }
        }
        size -= 1;
    }
    unsafe { slice.get_unchecked(..size) }
}

#[inline(always)]
fn truncate_naive_safe(slice: &[u8], mut size: usize) -> &[u8] {
    if size >= slice.len() {
        return slice;
    }
    while size > 0 {
        if is_char_boundary_at_safe(slice, size) {
            return unsafe { slice.get_unchecked(..size) };
        }
        size -= 1;
    }
    unsafe { slice.get_unchecked(..size) }
}

#[inline(always)]
fn truncate_naive_split(slice: &[u8], mut size: usize) -> &[u8] {
    while size > 0 {
        if let Some(ret) = split_at_char_boundary(slice, size) {
            return ret;
        }
        size -= 1;
    }
    &[]
}

#[inline(always)]
fn truncate_unrolled(slice: &[u8], mut size: usize) -> &[u8] {
    if size >= slice.len() {
        return slice;
    }
    unsafe {
        if is_char_boundary_at(slice, size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        if is_char_boundary_at(slice, size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        if is_char_boundary_at(slice, size) {
            return slice.get_unchecked(..size);
        }
        size -= 1;
        slice.get_unchecked(..size)
    }
}

#[inline(always)]
fn truncate_unrolled_safe(slice: &[u8], mut size: usize) -> &[u8] {
    if size >= slice.len() {
        return slice;
    }
    if is_char_boundary_at_safe(slice, size) {
        return unsafe { slice.get_unchecked(..size) };
    }
    size -= 1;
    if is_char_boundary_at_safe(slice, size) {
        return unsafe { slice.get_unchecked(..size) };
    }
    size -= 1;
    if is_char_boundary_at_safe(slice, size) {
        return unsafe { slice.get_unchecked(..size) };
    }
    size -= 1;
    return unsafe { slice.get_unchecked(..size) };
}

#[inline(always)]
fn split_at_char_boundary(arr: &[u8], index: usize) -> Option<&[u8]> {
    if index >= arr.len() {
        return Some(arr);
    }
    let (ret, bytes) = arr.split_at(index);
    bytes
        .get(0)
        .map(|it| (*it as i8) >= -0x40)
        .unwrap_or(false)
        .then_some(ret)
}

#[inline(always)]
unsafe fn is_char_boundary_at(arr: &[u8], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    (*arr.get_unchecked(index) as i8) >= -0x40
}

#[inline(always)]
fn is_char_boundary_at_safe(arr: &[u8], index: usize) -> bool {
    if index == 0 {
        return true;
    }
    match arr.get(index) {
        None => arr.len() == index,
        Some(a) => (*a as i8) >= -0x40,
    }
}

#[inline(always)]
const fn load_u32(slice: &[u8]) -> u32 {
    match slice {
        [a, b, c, d, ..] => u32::from_le_bytes([*a, *b, *c, *d]),
        [a, b, c] => u32::from_le_bytes([*a, *b, *c, 0]),
        [a, b] => u32::from_le_bytes([*a, *b, 0, 0]),
        [a] => u32::from_le_bytes([*a, 0, 0, 0]),
        [] => 0,
    }
}

#[inline(always)]
fn truncate_bits(slice: &[u8], size: usize) -> &[u8] {
    if let Some(bytes) = slice.get(size.saturating_sub(3)..) {
        let data = load_u32(bytes);
        let masked = data & 0xC0C0C0C0; // mask off only the two leftmost bis of each byte
        let zeroes = masked ^ 0x80808080; // the right values become zeroes, so we can count where the first 1 is that indicates the first non boundary
        let offset = zeroes.leading_zeros() / 8; // magic that gets the right offset
        unsafe { slice.get_unchecked(..size - offset as usize) }
    } else {
        slice
    }
}

fn truncate_benchmark_1byte(c: &mut Criterion) {
    let string = "abdefghijaaaaa".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&[u8], usize) -> &[u8]); 6] = [
        ("truncate_naive 1byte", truncate_naive),
        ("truncate_naive_safe 1byte", truncate_naive_safe),
        ("truncate_naive_split 1byte", truncate_naive_split),
        ("truncate_unrolled 1byte", truncate_unrolled),
        ("truncate_unrolled_safe 1byte", truncate_unrolled_safe),
        ("truncate_bits 1byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        for (name2, f2) in TESTS {
            assert_eq!(
                f(string.as_bytes(), 7),
                f2(string.as_bytes(), 7),
                "{} != {}",
                name,
                name2
            );
        }
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(string.as_bytes(), 7);
            })
        });
    }
}

fn truncate_benchmark_2byte(c: &mut Criterion) {
    let string = "ÅÅÅÅÅÅÅÅ".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&[u8], usize) -> &[u8]); 6] = [
        ("truncate_naive 2byte", truncate_naive),
        ("truncate_naive_safe 2byte", truncate_naive_safe),
        ("truncate_naive_split 2byte", truncate_naive_split),
        ("truncate_unrolled 2byte", truncate_unrolled),
        ("truncate_unrolled_safe 2byte", truncate_unrolled_safe),
        ("truncate_bits 2byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        for (name2, f2) in TESTS {
            assert_eq!(
                f(string.as_bytes(), 7),
                f2(string.as_bytes(), 7),
                "{} != {}",
                name,
                name2
            );
        }
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(string.as_bytes(), 7);
            })
        });
    }
}

fn truncate_benchmark_3byte(c: &mut Criterion) {
    let string = "⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠌⠍⠎⠏".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&[u8], usize) -> &[u8]); 6] = [
        ("truncate_naive 3byte", truncate_naive),
        ("truncate_naive_safe 3byte", truncate_naive_safe),
        ("truncate_naive_split 3byte", truncate_naive_split),
        ("truncate_unrolled 3byte", truncate_unrolled),
        ("truncate_unrolled_safe 3byte", truncate_unrolled_safe),
        ("truncate_bits 3byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        for (name2, f2) in TESTS {
            assert_eq!(
                f(string.as_bytes(), 7),
                f2(string.as_bytes(), 7),
                "{} != {}",
                name,
                name2
            );
        }
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(string.as_bytes(), 7);
            })
        });
    }
}

fn truncate_benchmark_4byte(c: &mut Criterion) {
    let string = "👍👍👍👍👍👍👍".repeat(thread_rng().gen_range(1..2));
    const TESTS: [(&str, fn(&[u8], usize) -> &[u8]); 6] = [
        ("truncate_naive 4byte", truncate_naive),
        ("truncate_naive_safe 4byte", truncate_naive_safe),
        ("truncate_naive_split 4byte", truncate_naive_split),
        ("truncate_unrolled 4byte", truncate_unrolled),
        ("truncate_unrolled_safe 4byte", truncate_unrolled_safe),
        ("truncate_bits 4byte", truncate_bits),
    ];
    for (name, f) in TESTS {
        let string = string.clone();
        for (name2, f2) in TESTS {
            assert_eq!(
                f(string.as_bytes(), 7),
                f2(string.as_bytes(), 7),
                "{} != {}",
                name,
                name2
            );
        }
        c.bench_function(name, move |b| {
            b.iter(|| {
                f(string.as_bytes(), 7);
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
criterion_main!(truncate,);
