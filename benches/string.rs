use arraystring::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};

fn string_clone_benchmark(c: &mut Criterion) {
    let string = String::from("abcdefghijklmnopqrst");
    c.bench_function("string clone", move |b| b.iter(|| string.clone()));
}

fn string_from_benchmark(c: &mut Criterion) {
    let string = String::from("uvwxyzaabbccddeeffgg");
    c.bench_function("string from", move |b| {
        b.iter(|| String::from(string.as_str()))
    });
}

fn small_clone_benchmark(c: &mut Criterion) {
    let string = SmallString::from_str_truncate("hhiijjkkllmmnnooppqq");
    c.bench_function("small clone", move |b| b.iter(|| string.clone()));
}

fn small_from_unchecked_benchmark(c: &mut Criterion) {
    let string = "rrssttuuvvwwxxyyzza";
    c.bench_function("small from unchecked", move |b| {
        b.iter(|| unsafe { SmallString::from_str_unchecked(&string) })
    });
}

fn small_from_truncate_benchmark(c: &mut Criterion) {
    let string = "bbbcccdddeeefffgggh";
    c.bench_function("small from truncate", move |b| {
        b.iter(|| SmallString::from_str_truncate(&string))
    });
}

fn small_try_from_benchmark(c: &mut Criterion) {
    let string = "iiijjjkkklllmmmnnnoo";
    c.bench_function("small try from", move |b| {
        b.iter(|| SmallString::try_from_str(&string))
    });
}

fn cache_clone_benchmark(c: &mut Criterion) {
    let string = CacheString::from_str_truncate("opppqqqrrrssstttuuuv");
    c.bench_function("cache clone", move |b| b.iter(|| string.clone()));
}

fn cache_from_unchecked_benchmark(c: &mut Criterion) {
    let string = "wwwxxxyyyzzzaaaabbbb";
    c.bench_function("cache from unchecked", move |b| {
        b.iter(|| unsafe { CacheString::from_str_unchecked(&string) })
    });
}

fn cache_from_truncate_benchmark(c: &mut Criterion) {
    let string = "ccccddddeeeeffffggggh";
    c.bench_function("cache from truncate", move |b| {
        b.iter(|| CacheString::from_str_truncate(&string))
    });
}

fn cache_try_from_benchmark(c: &mut Criterion) {
    let string = "iiiijjjjkkkkllllmmmmn";
    c.bench_function("cache try from", move |b| {
        b.iter(|| CacheString::try_from_str(&string))
    });
}

fn max_clone_benchmark(c: &mut Criterion) {
    let string = MaxString::from_str_truncate("ooopppqqqrrrssstttuu");
    c.bench_function("max clone", move |b| b.iter(|| string.clone()));
}

fn max_from_unchecked_benchmark(c: &mut Criterion) {
    let string = "vvvvwwwwxxxxyyyzzzza";
    c.bench_function("max from unchecked", move |b| {
        b.iter(|| unsafe { MaxString::from_str_unchecked(&string) })
    });
}

fn max_from_truncate_benchmark(c: &mut Criterion) {
    let string = "bbbbccccddddeeeeffff";
    c.bench_function("max from truncate", move |b| {
        b.iter(|| MaxString::from_str_truncate(&string))
    });
}

fn max_try_from_benchmark(c: &mut Criterion) {
    let string = "gggghhhhiiiijjjjkkkk";
    c.bench_function("max try from", move |b| {
        b.iter(|| MaxString::try_from_str(&string).unwrap())
    });
}

criterion_group!(string, string_clone_benchmark, string_from_benchmark,);
criterion_group!(
    small,
    small_clone_benchmark,
    small_try_from_benchmark,
    small_from_unchecked_benchmark,
    small_from_truncate_benchmark,
);
criterion_group!(
    cache,
    cache_clone_benchmark,
    cache_try_from_benchmark,
    cache_from_unchecked_benchmark,
    cache_from_truncate_benchmark,
);
criterion_group!(
    max,
    max_clone_benchmark,
    max_try_from_benchmark,
    max_from_unchecked_benchmark,
    max_from_truncate_benchmark,
);
criterion_main!(string, small, cache, max);
