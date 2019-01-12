use arraystring::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};

fn small_from_unchecked_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("small from unchecked", move |b| {
        b.iter(|| unsafe { SmallString::from_str_unchecked(&string) })
    });
}

fn small_from_truncate_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("small from truncate", move |b| {
        b.iter(|| SmallString::from_str_truncate(&string))
    });
}

fn small_try_from_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("small try from", move |b| {
        b.iter(|| SmallString::try_from_str(&string))
    });
}

fn cache_from_unchecked_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("cache from unchecked", move |b| {
        b.iter(|| unsafe { CacheString::from_str_unchecked(&string) })
    });
}

fn cache_from_truncate_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("cache from truncate", move |b| {
        b.iter(|| CacheString::from_str_truncate(&string))
    });
}

fn cache_try_from_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("cache try from", move |b| {
        b.iter(|| CacheString::try_from_str(&string))
    });
}

fn max_from_unchecked_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("max from unchecked", move |b| {
        b.iter(|| unsafe { MaxString::from_str_unchecked(&string) })
    });
}

fn max_from_truncate_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("max from truncate", move |b| {
        b.iter(|| MaxString::from_str_truncate(&string))
    });
}

fn max_try_from_benchmark(c: &mut Criterion) {
    let string = "0123456789";
    c.bench_function("max try from", move |b| {
        b.iter(|| MaxString::try_from_str(&string))
    });
}

criterion_group!(
    small,
    small_try_from_benchmark,
    small_from_unchecked_benchmark,
    small_from_truncate_benchmark,
);
criterion_group!(
    cache,
    cache_try_from_benchmark,
    cache_from_unchecked_benchmark,
    cache_from_truncate_benchmark,
);
criterion_group!(
    max,
    max_try_from_benchmark,
    max_from_unchecked_benchmark,
    max_from_truncate_benchmark,
);
criterion_main!(small, cache, max);
