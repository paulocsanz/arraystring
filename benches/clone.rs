use arraystring::prelude::*;
use criterion::{Criterion, criterion_main, criterion_group};

fn small_clone_benchmark(c: &mut Criterion) {
    let string = SmallString::from_str_truncate("0123456789".repeat(26));
    c.bench_function("small clone", move |b| b.iter(|| string.clone()));
}

fn cache_clone_benchmark(c: &mut Criterion) {
    let string = CacheString::from_str_truncate("0123456789".repeat(26));
    c.bench_function("cache clone", move |b| b.iter(|| string.clone()));
}

fn max_clone_benchmark(c: &mut Criterion) {
    let string = MaxString::from_str_truncate("0123456789".repeat(26));
    c.bench_function("max clone", move |b| b.iter(|| string.clone()));
}

criterion_group!(small, small_clone_benchmark);
criterion_group!(cache, cache_clone_benchmark);
criterion_group!(max, max_clone_benchmark);
criterion_main!(small, cache, max);
