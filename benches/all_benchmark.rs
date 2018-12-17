#[macro_use]
extern crate criterion;
extern crate arraystring;

use arraystring::prelude::*;
use criterion::Criterion;

fn truncate_benchmark(c: &mut Criterion) {
    let max_string = "0123456789".repeat(26);
    c.bench_function("cache from truncate", move |b| {
        b.iter(|| CacheString::from_str_truncate(&max_string))
    });
}

fn clone_benchmark(c: &mut Criterion) {
    let string = CacheString::from_str_truncate("0123456789".repeat(26));
    c.bench_function("cache clone", move |b| b.iter(|| string.clone()));
}

criterion_group!(benches, truncate_benchmark, clone_benchmark);
criterion_main!(benches);
