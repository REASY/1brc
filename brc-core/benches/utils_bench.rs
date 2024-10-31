use std::time::Duration;

use brc_core::{
    byte_to_string, byte_to_string_unsafe, parse_f64, to_scaled_integer,
    to_scaled_integer_branchless,
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

pub fn criterion_benchmark(c: &mut Criterion) {
    let str_as_bytes = "Thiès Lake Havasu City Yaoundé Petropavlovsk-Kamchatsky".as_bytes();
    let mut g = c.benchmark_group("Utils");
    g.bench_function("byte_to_string", |b| {
        b.iter(|| byte_to_string(black_box(str_as_bytes)))
    });

    g.bench_function("byte_to_string_unsafe", |b| {
        b.iter(|| byte_to_string_unsafe(black_box(str_as_bytes)))
    });

    g.bench_function("parse_f64", |b| b.iter(|| parse_f64(black_box("9.9"))));

    g.bench_function("get_as_scaled_integer", |b| {
        b.iter(|| to_scaled_integer(black_box("-99.9".as_bytes())))
    });

    g.bench_function("to_scaled_integer_branchless", |b| {
        b.iter(|| to_scaled_integer_branchless(black_box(0x65680A392E39392D)))
    });
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(20)).warm_up_time(Duration::from_secs(5));
  targets = criterion_benchmark
}
criterion_main!(benches);
