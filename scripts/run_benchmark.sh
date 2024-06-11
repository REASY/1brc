#!/usr/bin/env bash

set -eux

cargo build --release

run_app="./target/release/brc-app /media/ramdisk/measurements.txt"

# Running dummy readers to understand what is the fastest throughput
hyperfine --warmup 1 --runs 4 --export-markdown dummy.md "${run_app} 1 naive_line_by_line_dummy" \
  "${run_app} 1 parse_large_chunks_as_bytes_dummy" \
  "${run_app} 1 parse_large_chunks_as_i64_dummy" \
  "${run_app} 1 parse_large_chunks_simd_dummy"
  
function run_benchmark() {
    hyperfine --warmup 1 --runs 4 --export-markdown "${1}_threads.md" \
      "${run_app} $1 naive_line_by_line" \
      "${run_app} $1 naive_line_by_line_v2" \
      "${run_app} $1 parse_large_chunks_as_bytes" \
      "${run_app} $1 parse_large_chunks_as_i64" \
      "${run_app} $1 parse_large_chunks_as_i64_v2" \
      "${run_app} $1 parse_large_chunks_as_i64_unsafe" \
      "${run_app} $1 parse_large_chunks_as_i64_as_java" \
      "${run_app} $1 parse_large_chunks_simd" \
      "${run_app} $1 parse_large_chunks_simd_v1"
}

run_benchmark 1
run_benchmark 2
run_benchmark 4
run_benchmark 8
run_benchmark 16
run_benchmark 24
run_benchmark 32
run_benchmark 48