#!/usr/bin/env bash

set -eux

cargo build --release

# Running dummy readers to understand what is the fastest throughput
hyperfine --warmup 1 --runs 3 --export-markdown dummy.md './target/release/brc-app /media/ramdisk/measurements.txt 1 naive_line_by_line_dummy' \
  './target/release/brc-app /media/ramdisk/measurements.txt 1 parse_large_chunks_dummy' \
  './target/release/brc-app /media/ramdisk/measurements.txt 1 parse_large_chunks_simd_dummy'

# Running
hyperfine --warmup 1 --runs 3 --export-markdown 1_thread.md './target/release/brc-app /media/ramdisk/measurements.txt 1 naive_line_by_line' \
  './target/release/brc-app /media/ramdisk/measurements.txt 1 naive_line_by_line_v2' \
  './target/release/brc-app /media/ramdisk/measurements.txt 1 parse_large_chunks' \
  './target/release/brc-app /media/ramdisk/measurements.txt 1 parse_large_chunks_simd' \
  './target/release/brc-app /media/ramdisk/measurements.txt 1 parse_large_chunks_v1'

# Running 2 threads
hyperfine --warmup 1 --runs 3 --export-markdown 2_threads.md './target/release/brc-app /media/ramdisk/measurements.txt 2 naive_line_by_line_v2' \
  './target/release/brc-app /media/ramdisk/measurements.txt 2 parse_large_chunks' \
  './target/release/brc-app /media/ramdisk/measurements.txt 2 parse_large_chunks_simd' \
  './target/release/brc-app /media/ramdisk/measurements.txt 2 parse_large_chunks_v1'

# Running 4 threads
hyperfine --warmup 1 --runs 3 --export-markdown 4_threads.md './target/release/brc-app /media/ramdisk/measurements.txt 4 naive_line_by_line_v2' \
  './target/release/brc-app /media/ramdisk/measurements.txt 4 parse_large_chunks' \
  './target/release/brc-app /media/ramdisk/measurements.txt 4 parse_large_chunks_simd' \
  './target/release/brc-app /media/ramdisk/measurements.txt 4 parse_large_chunks_v1'

# Running 8 threads
hyperfine --warmup 1 --runs 3 --export-markdown 8_threads.md './target/release/brc-app /media/ramdisk/measurements.txt 8 naive_line_by_line_v2' \
  './target/release/brc-app /media/ramdisk/measurements.txt 8 parse_large_chunks' \
  './target/release/brc-app /media/ramdisk/measurements.txt 8 parse_large_chunks_simd' \
  './target/release/brc-app /media/ramdisk/measurements.txt 8 parse_large_chunks_v1'

# Running 16 threads
hyperfine --warmup 1 --runs 3 --export-markdown 16_threads.md './target/release/brc-app /media/ramdisk/measurements.txt 16 naive_line_by_line_v2' \
  './target/release/brc-app /media/ramdisk/measurements.txt 16 parse_large_chunks' \
  './target/release/brc-app /media/ramdisk/measurements.txt 16 parse_large_chunks_simd' \
  './target/release/brc-app /media/ramdisk/measurements.txt 16 parse_large_chunks_v1'

# Running 32 threads
hyperfine --warmup 1 --runs 3 --export-markdown 32_threads.md './target/release/brc-app /media/ramdisk/measurements.txt 32 naive_line_by_line_v2' \
  './target/release/brc-app /media/ramdisk/measurements.txt 32 parse_large_chunks' \
  './target/release/brc-app /media/ramdisk/measurements.txt 32 parse_large_chunks_simd' \
  './target/release/brc-app /media/ramdisk/measurements.txt 32 parse_large_chunks_v1'

# Running 48 threads
hyperfine --warmup 1 --runs 3 --export-markdown 48_threads.md './target/release/brc-app /media/ramdisk/measurements.txt 48 naive_line_by_line_v2' \
  './target/release/brc-app /media/ramdisk/measurements.txt 48 parse_large_chunks' \
  './target/release/brc-app /media/ramdisk/measurements.txt 48 parse_large_chunks_simd' \
  './target/release/brc-app /media/ramdisk/measurements.txt 48 parse_large_chunks_v1'