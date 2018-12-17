#[macro_use]
extern crate criterion;
extern crate arraystring;

use arraystring::prelude::*;
use criterion::{Criterion, Fun};
use std::sync::Arc;

fn from_benchmark(c: &mut Criterion) {
    let max_string: &'static str = Box::leak("0123456789".repeat(26).into_boxed_str());
    let array = Fun::new("Array", move |b, &()| {
        b.iter(|| SmallString::from_str_truncate(max_string))
    });
    let heap = Fun::new("Heap", move |b, &()| b.iter(|| String::from(max_string)));
    let functions = vec![array, heap];
    c.bench_functions("From", functions, ());
}

fn clone_benchmark(c: &mut Criterion) {
    let max_string = "0123456789".repeat(26);
    let array = SmallString::from_str_truncate(&max_string);
    let heap = String::from(max_string.as_str());
    let arc = Arc::new(heap.clone());

    let array = Fun::new("Array", move |b, ()| b.iter(|| array.clone()));
    let heap = Fun::new("Heap", move |b, ()| b.iter(|| heap.clone()));
    let arc = Fun::new("Heap", move |b, ()| b.iter(|| arc.clone()));
    let functions = vec![array, heap, arc];
    c.bench_functions("Clone", functions, ());
}

criterion_group!(benches, from_benchmark, clone_benchmark);
criterion_main!(benches);
