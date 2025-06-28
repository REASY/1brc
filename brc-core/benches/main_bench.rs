use std::io::{BufReader, Cursor};
use std::time::Duration;

use brc_core::{
    naive_line_by_line, naive_line_by_line_dummy, naive_line_by_line_v2,
    parse_large_chunks_as_bytes, parse_large_chunks_as_bytes_dummy, parse_large_chunks_as_i64,
    parse_large_chunks_simd, parse_large_chunks_simd_dummy, parse_large_chunks_simd_v2,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn get_buf_reader(bytes: &[u8]) -> BufReader<Cursor<&[u8]>> {
    BufReader::with_capacity(64 * 1024 * 1024, Cursor::new(bytes))
}

fn naive_line_by_line_dummy_benchmark(bytes: &[u8]) {
    let r = naive_line_by_line_dummy(get_buf_reader(bytes), 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}
fn parse_large_chunks_simd_dummy_benchmark(bytes: &[u8]) {
    let r =
        parse_large_chunks_simd_dummy(get_buf_reader(bytes), 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}
fn parse_large_chunks_as_bytes_dummy_benchmark(bytes: &[u8]) {
    let r = parse_large_chunks_as_bytes_dummy(
        get_buf_reader(bytes),
        0,
        (bytes.len() as u64) - 1,
        false,
    );
    black_box(r);
}
fn naive_line_by_line_benchmark(bytes: &[u8]) {
    let r = naive_line_by_line(get_buf_reader(bytes), 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}
fn naive_line_by_line_v2_benchmark(bytes: &[u8]) {
    let r = naive_line_by_line_v2(get_buf_reader(bytes), 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}
fn parse_large_chunks_as_bytes_benchmark(bytes: &[u8]) {
    let r = parse_large_chunks_as_bytes(get_buf_reader(bytes), 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}
fn parse_large_chunks_simd_benchmark(bytes: &[u8]) {
    let r = parse_large_chunks_simd(get_buf_reader(bytes), 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}
fn parse_large_chunks_as_i64_benchmark(bytes: &[u8]) {
    let r = parse_large_chunks_as_i64(get_buf_reader(bytes), 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}
fn parse_large_chunks_simd_v2_benchmark(bytes: &[u8]) {
    let r = parse_large_chunks_simd_v2(get_buf_reader(bytes), 0, (bytes.len() as u64) - 1, false);
    black_box(r);
}

const N: usize = 10;

pub fn criterion_benchmark(c: &mut Criterion) {
    let bytes = {
        let b = include_bytes!("../test_resources/sample.txt");
        let mut v: Vec<u8> = Vec::with_capacity(N * b.len());
        for _ in 0..N {
            v.append(&mut b.to_vec());
        }
        v
    };

    let mut g = c.benchmark_group("dummy reader");
    g.throughput(Throughput::Bytes(bytes.len() as u64));

    g.bench_with_input(
        BenchmarkId::new("naive_line_by_line_dummy", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| naive_line_by_line_dummy_benchmark(bytes)),
    );
    g.bench_with_input(
        BenchmarkId::new("parse_large_chunks_as_bytes_dummy", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| parse_large_chunks_as_bytes_dummy_benchmark(bytes)),
    );
    g.bench_with_input(
        BenchmarkId::new("parse_large_chunks_simd_dummy", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| parse_large_chunks_simd_dummy_benchmark(bytes)),
    );
    g.finish();

    let mut g = c.benchmark_group("different implementation");
    g.throughput(Throughput::Bytes(bytes.len() as u64));

    g.bench_with_input(
        BenchmarkId::new("naive_line_by_line", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| naive_line_by_line_benchmark(bytes)),
    );
    g.bench_with_input(
        BenchmarkId::new("naive_line_by_line_v2", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| naive_line_by_line_v2_benchmark(bytes)),
    );
    g.bench_with_input(
        BenchmarkId::new("parse_large_chunks_as_bytes", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| parse_large_chunks_as_bytes_benchmark(bytes)),
    );
    g.bench_with_input(
        BenchmarkId::new("parse_large_chunks_simd", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| parse_large_chunks_simd_benchmark(bytes)),
    );
    g.bench_with_input(
        BenchmarkId::new("parse_large_chunks_as_i64", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| parse_large_chunks_as_i64_benchmark(bytes)),
    );
    g.bench_with_input(
        BenchmarkId::new("parse_large_chunks_simd_v2", bytes.len()),
        bytes.as_slice(),
        |b, bytes| b.iter(|| parse_large_chunks_simd_v2_benchmark(bytes)),
    );
    g.finish();
}

criterion_group! {
  name = benches;
  config =
Criterion::default().measurement_time(Duration::from_secs(20)).
warm_up_time(Duration::from_secs(5));   targets = criterion_benchmark
}
criterion_main!(benches);
