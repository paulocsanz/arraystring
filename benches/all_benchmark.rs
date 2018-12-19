#[macro_use]
extern crate criterion;
extern crate arraystring;

use arraystring::prelude::*;
use criterion::Criterion;

fn small_from_unchecked_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("small from unchecked", move |b| {
        b.iter(|| unsafe { SmallString::from_str_unchecked(&max_string) })
    });
}

fn small_from_truncate_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("small from truncate", move |b| {
        b.iter(|| SmallString::from_str_truncate(&max_string))
    });
}

fn small_try_from_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("small try from", move |b| {
        b.iter(|| SmallString::try_from_str(&max_string))
    });
}

fn cache_from_unchecked_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("cache from unchecked", move |b| {
        b.iter(|| unsafe { CacheString::from_str_unchecked(&max_string) })
    });
}

fn cache_from_truncate_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("cache from truncate", move |b| {
        b.iter(|| CacheString::from_str_truncate(&max_string))
    });
}

fn cache_try_from_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("cache try from", move |b| {
        b.iter(|| CacheString::try_from_str(&max_string))
    });
}

fn max_from_unchecked_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("max from unchecked", move |b| {
        b.iter(|| unsafe { InlinableString::from_str_unchecked(&max_string) })
    });
}

fn max_from_truncate_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("max from truncate", move |b| {
        b.iter(|| InlinableString::from_str_truncate(&max_string))
    });
}

fn max_try_from_benchmark(c: &mut Criterion) {
    let max_string = "0123456789";
    c.bench_function("max try from", move |b| {
        b.iter(|| InlinableString::try_from_str(&max_string))
    });
}

fn small_clone_benchmark(c: &mut Criterion) {
    let string = SmallString::from_str_truncate("0123456789".repeat(26));
    c.bench_function("small clone", move |b| b.iter(|| string.clone()));
}

fn cache_clone_benchmark(c: &mut Criterion) {
    let string = CacheString::from_str_truncate("0123456789".repeat(26));
    c.bench_function("cache clone", move |b| b.iter(|| string.clone()));
}

fn max_clone_benchmark(c: &mut Criterion) {
    let string = InlinableString::from_str_truncate("0123456789".repeat(26));
    c.bench_function("max clone", move |b| b.iter(|| string.clone()));
}

criterion_group!(
    small,
    small_try_from_benchmark,
    small_from_unchecked_benchmark,
    small_from_truncate_benchmark,
    small_clone_benchmark
);
criterion_group!(
    cache,
    cache_try_from_benchmark,
    cache_from_unchecked_benchmark,
    cache_from_truncate_benchmark,
    cache_clone_benchmark
);
criterion_group!(
    max,
    max_try_from_benchmark,
    max_from_unchecked_benchmark,
    max_from_truncate_benchmark,
    max_clone_benchmark
);
criterion_main!(small, cache, max);
