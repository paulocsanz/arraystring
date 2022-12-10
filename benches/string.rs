use arraystring::prelude::*;
use arrayvec::ArrayString as ArrayVecString;
use criterion::{criterion_group, criterion_main, Criterion};
use inlinable_string::{InlinableString, StringExt};
use rand::distributions::Alphanumeric;
use rand::{Rng, thread_rng};
use smallstring::SmallString as SmallVecString;

fn string_clone_benchmark(c: &mut Criterion) {
    let string = String::from("abcdefghijklmnopqrst");
    c.bench_function("string clone", move |b| b.iter(|| string.clone()));
}

fn string_from_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("string from", move |b| {
        b.iter(|| String::from(&rand_string))
    });
}

fn string_push_str_benchmark(c: &mut Criterion) {
    let mut string = String::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("string push str", move |b| {
        b.iter(|| {
            string.push_str(&rand_string);
            string.clear();
            string.shrink_to_fit();
        })
    });
}

fn inlinable_clone_benchmark(c: &mut Criterion) {
    let string = InlinableString::from("hcuahdaidshdaisuhda");
    c.bench_function("inlinable clone", move |b| b.iter(|| string.clone()));
}

fn inlinable_from_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("inlinable from", move |b| {
        b.iter(|| InlinableString::from(rand_string.as_str()))
    });
}

fn inlinable_push_str_benchmark(c: &mut Criterion) {
    let mut string = InlinableString::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("inlinable push str", move |b| {
        b.iter(|| {
            string.push_str(&rand_string);
            string.clear();
            string.shrink_to_fit();
        })
    });
}

fn arrayvec_clone_benchmark(c: &mut Criterion) {
    let string = ArrayVecString::<[u8; 23]>::from("fhuehifhsaudhaisdha");
    c.bench_function("arrayvec string clone", move |b| b.iter(|| string.clone()));
}

fn arrayvec_from_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("arrayvec string from", move |b| {
        b.iter(|| ArrayVecString::<[u8; 23]>::from(&rand_string))
    });
}

fn arrayvec_push_str_benchmark(c: &mut Criterion) {
    let mut string = ArrayVecString::<[u8; 23]>::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("arrayvec string push str", move |b| {
        b.iter(|| {
            string.push_str(&rand_string);
            string.clear();
        })
    });
}

fn smallvecstring_clone_benchmark(c: &mut Criterion) {
    let string = SmallVecString::<[u8;20]>::from("xhduibabicemlatdhue");
    c.bench_function("smallvecstring clone", move |b| b.iter(|| string.clone()));
}

fn smallvecstring_from_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("smallvecstring from", move |b| {
        b.iter(|| SmallVecString::<[u8;20]>::from(rand_string.as_str()))
    });
}

fn small_clone_benchmark(c: &mut Criterion) {
    let string = SmallString::from_str_truncate("hhiijjkkllmmneeeepqq");
    c.bench_function("small clone", move |b| b.iter(|| string.clone()));
}

fn small_from_truncate_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("small from truncate", move |b| {
        b.iter(|| SmallString::from_str_truncate(&rand_string))
    });
}

fn small_try_from_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("small try from", move |b| {
        b.iter(|| SmallString::try_from_str(&rand_string))
    });
}

fn small_push_str_benchmark(c: &mut Criterion) {
    let mut string = SmallString::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("small push str truncate", move |b| {
        b.iter(|| {
            string.push_str(&rand_string);
            string.clear();
        })
    });
}

fn small_try_push_str_benchmark(c: &mut Criterion) {
    let mut string = SmallString::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("small try push str", move |b| {
        b.iter(|| {
            string.try_push_str(&rand_string).unwrap();
            string.clear();
        })
    });
}

fn cache_clone_benchmark(c: &mut Criterion) {
    let string = CacheString::from_str_truncate("opppqqqrrrssstttuuuv");
    c.bench_function("cache clone", move |b| b.iter(|| string.clone()));
}

fn cache_from_truncate_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("cache from truncate", move |b| {
        b.iter(|| CacheString::from_str_truncate(&rand_string))
    });
}

fn cache_try_from_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("cache try from", move |b| {
        b.iter(|| CacheString::try_from_str(&rand_string))
    });
}

fn cache_push_str_benchmark(c: &mut Criterion) {
    let mut string = CacheString::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("cache push str truncate", move |b| {
        b.iter(|| {
            string.push_str(&rand_string);
            string.clear();
        })
    });
}

fn cache_try_push_str_benchmark(c: &mut Criterion) {
    let mut string = CacheString::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("cache try push str", move |b| {
        b.iter(|| {
            string.try_push_str(&rand_string).unwrap();
            string.clear();
        })
    });
}

fn max_clone_benchmark(c: &mut Criterion) {
    let string = MaxString::from_str_truncate("ooopppqqqrrrssstttuu");
    c.bench_function("max clone", move |b| b.iter(|| string.clone()));
}

fn max_from_truncate_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("max from truncate", move |b| {
        b.iter(|| MaxString::from_str_truncate(&rand_string))
    });
}

fn max_try_from_benchmark(c: &mut Criterion) {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("max try from", move |b| {
        b.iter(|| MaxString::try_from_str(&rand_string).unwrap())
    });
}

fn max_push_str_benchmark(c: &mut Criterion) {
    let mut string = MaxString::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("max push str truncate", move |b| {
        b.iter(|| {
            string.push_str(&rand_string);
            string.clear();
        })
    });
}

fn max_try_push_str_benchmark(c: &mut Criterion) {
    let mut string = MaxString::default();
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(19).map(char::from).collect();
    c.bench_function("max try push str", move |b| {
        b.iter(|| {
            string.try_push_str(&rand_string).unwrap();
            string.clear();
        })
    });
}

criterion_group!(
    string,
    string_clone_benchmark,
    string_from_benchmark,
    string_push_str_benchmark
);
criterion_group!(
    inlinable,
    inlinable_clone_benchmark,
    inlinable_from_benchmark,
    inlinable_push_str_benchmark
);
criterion_group!(
    arrayvec,
    arrayvec_clone_benchmark,
    arrayvec_from_benchmark,
    arrayvec_push_str_benchmark,
);
criterion_group!(
    smallvecstring,
    smallvecstring_clone_benchmark,
    smallvecstring_from_benchmark,
);
criterion_group!(
    small,
    small_clone_benchmark,
    small_try_from_benchmark,
    small_from_truncate_benchmark,
    small_try_push_str_benchmark,
    small_push_str_benchmark,
);
criterion_group!(
    cache,
    cache_clone_benchmark,
    cache_try_from_benchmark,
    cache_from_truncate_benchmark,
    cache_try_push_str_benchmark,
    cache_push_str_benchmark,
);
criterion_group!(
    max,
    max_clone_benchmark,
    max_try_from_benchmark,
    max_from_truncate_benchmark,
    max_try_push_str_benchmark,
    max_push_str_benchmark,
);
criterion_main!(
    string,
    arrayvec,
    inlinable,
    smallvecstring,
    small,
    cache,
    max
);
