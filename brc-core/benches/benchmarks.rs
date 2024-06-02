use brc_core::{
    byte_to_string, byte_to_string_unsafe, custom_parse_f64, improved_impl_v3,
    improved_impl_v3_dummy_simd_search, improved_impl_v4, naive_line_by_line,
    naive_line_by_line_dummy, naive_line_by_line_v2, parse_f64, parse_large_chunks_dummy,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::io::{BufReader, Cursor};
use std::time::Duration;

fn naive_line_by_line_dummy_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = naive_line_by_line_dummy(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

fn naive_line_by_line_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = naive_line_by_line(rdr, 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

fn improved_impl_v1_benchmark(bytes: &[u8]) {
    assert_ne!(0, bytes.len());

    let rdr = BufReader::with_capacity(64 * 1024, Cursor::new(bytes));
    let r = naive_line_by_line_v2(rdr, 0, (bytes.len() as u64) - 1, false);
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
    let r = parse_large_chunks_dummy(rdr, 0, (bytes.len() as u64) - 1, false);
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

    let mut g = c.benchmark_group("dummy reader of 38049 lines");
    g.throughput(Throughput::Bytes(bytes.len() as u64));

    g.bench_function("naive_line_by_line_dummy", |b| {
        b.iter(|| naive_line_by_line_dummy_benchmark(black_box(bytes.as_slice())))
    });
    g.bench_function("improved_impl_v3_dummy", |b| {
        b.iter(|| improved_impl_v3_dummy_benchmark(black_box(bytes.as_slice())))
    });
    g.finish();

    let mut g = c.benchmark_group("different implementation");
    g.throughput(Throughput::Bytes(bytes.len() as u64));

    g.bench_function("naive_line_by_line for 38049 lines", |b| {
        b.iter(|| naive_line_by_line_benchmark(black_box(bytes.as_slice())))
    });

    g.bench_function("improved_impl_v1 for 38049 lines", |b| {
        b.iter(|| improved_impl_v1_benchmark(black_box(bytes.as_slice())))
    });

    g.bench_function("improved_impl_v3_benchmark for 38049 lines", |b| {
        b.iter(|| improved_impl_v3_benchmark(black_box(bytes.as_slice())))
    });

    g.bench_function("improved_impl_v4_benchmark for 38049 lines", |b| {
        b.iter(|| improved_impl_v4_benchmark(black_box(bytes.as_slice())))
    });
    g.finish();

    let mut g = c.benchmark_group("Utils");
    g.bench_function("byte_to_string", |b| {
        b.iter(|| byte_to_string(black_box(str_as_bytes)))
    });

    g.bench_function("byte_to_string_unsafe", |b| {
        b.iter(|| byte_to_string_unsafe(black_box(str_as_bytes)))
    });

    g.bench_function("parse_f64", |b| b.iter(|| parse_f64(black_box("9.9"))));

    g.bench_function("custom_parse_f64", |b| {
        b.iter(|| custom_parse_f64(black_box("9.9")))
    });
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(20));
  targets = criterion_benchmark
}
criterion_main!(benches);
