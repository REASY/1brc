# 1 Billion Row Challenge in Rust

This repository explores progressively more specialized implementations of the
[1 Billion Row Challenge](https://github.com/gunnarmorling/1brc), from ordinary
line parsing to word-at-a-time parsing, vectorized delimiter discovery, compact
fingerprint tables, and batched SIMD temperature aggregation.

## Reproducing the benchmark

The [benchmark script](scripts/run_benchmark.sh) requires the measurements file
as its first argument:

```bash
scripts/run_benchmark.sh /path/to/measurements.txt
```

Results are written to `benchmark-results/`. A different output directory can
be supplied as the second argument:

```bash
scripts/run_benchmark.sh /path/to/measurements.txt /tmp/1brc-results
```

The defaults are one warmup, four measured runs, and scaling at 1, 2, 4, 8, 16,
24, and 32 threads. They can be overridden without editing the script:

```bash
HYPERFINE_WARMUP=2 \
HYPERFINE_RUNS=10 \
THREAD_COUNTS="1 2 4 8 16" \
scripts/run_benchmark.sh /path/to/measurements.txt
```

The script builds the release binary, runs all implementations at one thread,
and scales the previous baseline implementations together with all newer
table/SIMD contenders. It requires
[hyperfine](https://github.com/sharkdp/hyperfine) in `PATH`.

## Benchmark environment

Results below were collected on 2026-07-10 with:

- AMD Ryzen 9 9950X: 16 cores, 32 hardware threads, performance governor
- 59 GiB memory visible to Linux
- Linux 7.0.0-27-generic, x86-64
- `rustc 1.99.0-nightly (af3d95584 2026-07-09)`, LLVM 22.1.8
- release profile with fat LTO, one codegen unit, and `-C target-cpu=native`
- hyperfine 1.20.0, one warmup and four measured runs
- 1,000,000,000 rows, 13,795,343,590 bytes (13.80 GB)
- input on ext4/NVMe and resident in the page cache after warmup

Times include process startup, parsing, aggregation, sorting, and printing the
result. No CPU affinity was applied. Values are hyperfine mean ± sample standard
deviation; the min and max columns cover the four measured runs.

## Results

The best result is `parse_large_chunks_memchr_table` at **0.962 ± 0.010 s with
16 threads**. That is approximately 14.34 GB/s and 1.04 billion records/s. More
workers do not help on this machine: it regresses to 0.994 s at 24 threads and
1.089 s at 32 threads.

At one thread, the same implementation takes **5.638 ± 0.024 s**, 1.40× faster
than the previous `parse_large_chunks_as_i64_as_java` winner and 1.14× faster
than batched SIMD temperature parsing. The simpler memchr/table loop remains
faster than the broader SIMD designs at every useful thread count.

### Scaling: previous and new contenders

The cells below are mean wall-clock seconds. The first two rows are the previous
scaling baselines; the remaining four are the newer table/SIMD variants.

| Implementation                              |  1 thread | 2 threads | 4 threads | 8 threads | 16 threads | 24 threads | 32 threads |
|:--------------------------------------------|----------:|----------:|----------:|----------:|-----------:|-----------:|-----------:|
| `parse_large_chunks_as_i64_v2`              |     9.750 |     4.929 |     2.622 |     1.536 |      1.123 |      1.085 |      1.132 |
| `parse_large_chunks_as_i64_as_java`         |     7.876 |     3.904 |     2.068 |     1.309 |      1.027 |      1.030 |      1.128 |
| `parse_large_chunks_memchr_table`           | **5.638** | **2.909** | **1.648** | **1.096** |  **0.962** |  **0.994** |  **1.089** |
| `parse_large_chunks_std_simd_table`         |     7.075 |     3.656 |     2.044 |     1.272 |      1.059 |      1.032 |      1.091 |
| `parse_large_chunks_simd_temperature_table` |     6.453 |     3.323 |     1.825 |     1.207 |      1.047 |      1.059 |      1.150 |
| `parse_large_chunks_full_simd_table`        |     7.178 |     3.729 |     2.010 |     1.290 |      1.013 |      1.048 |      1.113 |

The scaling curve flattens after eight threads and reaches its minimum at the 16
physical cores. At 24 and 32 workers, increased scheduling and kernel overhead
outweigh the remaining parallelism.

### All implementations at one thread

| Implementation                              |          Mean [s] |   Min [s] |   Max [s] | Relative to fastest |
|:--------------------------------------------|------------------:|----------:|----------:|--------------------:|
| `naive_line_by_line`                        |    45.180 ± 8.500 |    40.821 |    57.927 |               8.01× |
| `naive_line_by_line_v2`                     |    26.651 ± 0.453 |    26.193 |    27.245 |               4.73× |
| `parse_large_chunks_as_bytes`               |    13.640 ± 0.079 |    13.561 |    13.741 |               2.42× |
| `parse_large_chunks_as_i64`                 |     9.662 ± 0.045 |     9.625 |     9.728 |               1.71× |
| `parse_large_chunks_as_i64_v2`              |     9.750 ± 0.084 |     9.662 |     9.864 |               1.73× |
| `parse_large_chunks_as_i64_unsafe`          |    10.152 ± 0.034 |    10.104 |    10.179 |               1.80× |
| `parse_large_chunks_as_i64_as_java`         |     7.876 ± 0.478 |     7.603 |     8.592 |               1.40× |
| `parse_large_chunks_simd`                   |    15.624 ± 1.712 |    13.411 |    17.589 |               2.77× |
| `parse_large_chunks_simd_v1`                |    12.966 ± 0.066 |    12.898 |    13.048 |               2.30× |
| `parse_large_chunks_simd_v2`                |    18.586 ± 0.052 |    18.515 |    18.638 |               3.30× |
| `parse_large_chunks_memchr_table`           | **5.638 ± 0.024** | **5.616** | **5.662** |           **1.00×** |
| `parse_large_chunks_std_simd_table`         |     7.075 ± 0.121 |     6.961 |     7.237 |               1.25× |
| `parse_large_chunks_simd_temperature_table` |     6.453 ± 0.031 |     6.413 |     6.489 |               1.14× |
| `parse_large_chunks_full_simd_table`        |     7.178 ± 0.008 |     7.171 |     7.189 |               1.27× |

Hyperfine detected statistical outliers in `naive_line_by_line` and
`parse_large_chunks_as_i64_as_java`; their means should be interpreted with the
reported spread. The optimized winner was stable across all four measurements.

### Single-thread parsing-only implementations

The dummy variants parse every record but omit the station aggregation. They
help separate delimiter/temperature parsing cost from table-update cost.

| Implementation                      |          Mean [s] |   Min [s] |   Max [s] | Relative to fastest |
|:------------------------------------|------------------:|----------:|----------:|--------------------:|
| `naive_line_by_line_dummy`          |    20.189 ± 0.033 |    20.166 |    20.238 |               4.90× |
| `parse_large_chunks_as_bytes_dummy` |    10.811 ± 0.006 |    10.805 |    10.817 |               2.62× |
| `parse_large_chunks_as_i64_dummy`   |     7.354 ± 0.008 |     7.345 |     7.363 |               1.78× |
| `parse_large_chunks_simd_dummy`     | **4.123 ± 0.024** | **4.107** | **4.159** |           **1.00×** |

The dummy results show that SIMD delimiter parsing itself is effective. The
broader SIMD implementations lose their advantage in lookup, batching, and
gather/scatter overhead around that fast parser.

## Correctness scope of the fingerprint-table variants

The four `*_table` implementations are deliberately specialized for the 1BRC
workload. They use a compact fingerprint of the station length plus its first
and last eight bytes and treat fingerprint equality as station equality. Linear
probing handles table-index collisions, but two distinct station names with an
identical 64-bit fingerprint would be merged. This removes a variable-length key
comparison from every record and is safe for the challenge's known station set;
it is not a general-purpose hash table for arbitrary input.
