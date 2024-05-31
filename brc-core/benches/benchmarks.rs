use brc_core::{
    byte_to_string, byte_to_string_unsafe, custom_parse_f64, improved_impl_v1, improved_impl_v2,
    improved_impl_v3, improved_impl_v3_dummy, improved_impl_v3_dummy_simd_search, improved_impl_v4,
    naive_impl, parse_f64,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::{BufReader, Cursor};

fn naive_impl_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = naive_impl(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

fn improved_impl_v1_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = improved_impl_v1(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

fn improved_impl_v2_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = improved_impl_v2(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

fn improved_impl_v3_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = improved_impl_v3(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

fn improved_impl_v3_dummy_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = improved_impl_v3_dummy(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

fn improved_impl_v3_dummy_simd_search_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = improved_impl_v3_dummy_simd_search(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

fn improved_impl_v4_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = improved_impl_v4(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let bytes = include_bytes!("../test_resources/sample.txt").to_vec();
    let str_as_bytes = "Thiès Lake Havasu City Yaoundé Petropavlovsk-Kamchatsky".as_bytes();

    c.bench_function("improved_impl_v3_dummy_benchmark for 38049 lines", |b| {
        b.iter(|| improved_impl_v3_dummy_benchmark(bytes.as_slice()))
    });

    c.bench_function(
        "improved_impl_v3_dummy_simd_search_benchmark for 38049 lines",
        |b| b.iter(|| improved_impl_v3_dummy_simd_search_benchmark(bytes.as_slice())),
    );

    c.bench_function("naive_impl_benchmark for 38049 lines", |b| {
        b.iter(|| naive_impl_benchmark(bytes.as_slice()))
    });

    c.bench_function("improved_impl_v1 for 38049 lines", |b| {
        b.iter(|| improved_impl_v1_benchmark(bytes.as_slice()))
    });

    c.bench_function("improved_impl_v2_benchmark for 38049 lines", |b| {
        b.iter(|| improved_impl_v2_benchmark(bytes.as_slice()))
    });

    c.bench_function("improved_impl_v3_benchmark for 38049 lines", |b| {
        b.iter(|| improved_impl_v3_benchmark(bytes.as_slice()))
    });

    c.bench_function("improved_impl_v4_benchmark for 38049 lines", |b| {
        b.iter(|| improved_impl_v4_benchmark(bytes.as_slice()))
    });

    c.bench_function("byte_to_string", |b| {
        b.iter(|| byte_to_string(black_box(str_as_bytes)))
    });

    c.bench_function("byte_to_string_unsafe", |b| {
        b.iter(|| byte_to_string_unsafe(black_box(str_as_bytes)))
    });

    c.bench_function("parse_f64", |b| b.iter(|| parse_f64(black_box("9.9"))));

    c.bench_function("custom_parse_f64", |b| {
        b.iter(|| custom_parse_f64(black_box("9.9")))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
