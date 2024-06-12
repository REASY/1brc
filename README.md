1 Billion Row Challenge in Rust
==========

# Environment for the benchmark

- 48 Intel vCPUs / 96 GB Memory / 600 GB Disk, dedicated CPU-optimized DigitalOcean instance with Premium Intel
  CPU, [c-48-intel](https://docs.digitalocean.com/products/droplets/details/pricing/)
- Rust compiler rustc 1.78.0 (9b00956e5 2024-04-29), x86_64-unknown-linux-gnu, LLVM version: 18.1.2
- Java 21.0.3 2024-04-16 LTS, Java HotSpot(TM) 64-Bit Server VM Oracle GraalVM 21.0.3+7.1 (build
  21.0.3+7-LTS-jvmci-23.1-b37, mixed mode, sharing)

# Reference Rust solution [tumdum/1brc](https://github.com/tumdum/1brc)

The results obtained by running [scripts/run_benchmark_original.sh](scripts/run_benchmark_original.sh). It slightly
modifies the original code to allow passing number of cores and also makes sure the project is built with native CPU
support to get max performance.

| Number of threads |       Mean [s] | Min [s] | Max [s] |
|:------------------|---------------:|--------:|--------:|
| 1                 | 42.898 ± 0.068 |  42.830 |  42.992 |
| 2                 | 21.718 ± 0.019 |  21.703 |  21.745 |
| 4                 | 10.955 ± 0.046 |  10.904 |  11.017 |
| 8                 |  5.686 ± 0.036 |   5.652 |   5.724 |
| 16                |  3.024 ± 0.019 |   3.006 |   3.050 |
| 24                |  2.209 ± 0.029 |   2.168 |   2.231 |
| 32                |  2.185 ± 0.013 |   2.176 |   2.205 |
| 48                |  1.722 ± 0.007 |   1.711 |   1.726 |

# My results

The results obtained by running [scripts/run_benchmark.sh](scripts/run_benchmark.sh).

| Implementation                    | Number of threads |       Mean [s] | Min [s] | Max [s] |
|:----------------------------------|-------------------|---------------:|--------:|--------:|
| parse_large_chunks_as_i64_v2      | 1                 | 19.797 ± 0.031 |  19.764 |  19.830 |
| parse_large_chunks_as_i64_as_java | 1                 | 16.848 ± 0.050 |  16.784 |  16.900 |
| parse_large_chunks_as_i64_v2      | 2                 | 10.010 ± 0.016 |   9.990 |  10.030 |
| parse_large_chunks_as_i64_as_java | 2                 |  8.570 ± 0.010 |   8.558 |   8.579 |
| parse_large_chunks_as_i64_v2      | 4                 |  5.077 ± 0.010 |   5.065 |   5.090 |
| parse_large_chunks_as_i64_as_java | 4                 |  4.373 ± 0.005 |   4.368 |   4.380 |
| parse_large_chunks_as_i64_v2      | 8                 |  2.650 ± 0.010 |   2.639 |   2.661 |
| parse_large_chunks_as_i64_as_java | 8                 |  2.299 ± 0.009 |   2.286 |   2.306 |
| parse_large_chunks_as_i64_v2      | 16                |  1.496 ± 0.014 |   1.478 |   1.507 |
| parse_large_chunks_as_i64_as_java | 16                |  1.321 ± 0.006 |   1.312 |   1.326 |
| parse_large_chunks_as_i64_v2      | 24                |  1.148 ± 0.009 |   1.137 |   1.156 |
| parse_large_chunks_as_i64_as_java | 24                |  1.057 ± 0.058 |   1.002 |   1.138 |
| parse_large_chunks_as_i64_v2      | 32                |  1.232 ± 0.019 |   1.214 |   1.259 |
| parse_large_chunks_as_i64_as_java | 32                |  1.127 ± 0.009 |   1.116 |   1.138 |
| parse_large_chunks_as_i64_v2      | 48                |  1.223 ± 0.012 |   1.210 |   1.238 |
| parse_large_chunks_as_i64_as_java | 48                |  1.174 ± 0.008 |   1.163 |   1.180 |

## Single-thread dummy implementations to understand how fast it can be

| Implementation                    |       Mean [s] | Min [s] | Max [s] |
|:----------------------------------|---------------:|--------:|--------:|
| naive_line_by_line_dummy          | 44.508 ± 0.370 |  44.191 |  45.042 |
| parse_large_chunks_as_bytes_dummy | 25.021 ± 0.059 |  24.941 |  25.081 |
| parse_large_chunks_as_i64_dummy   | 16.163 ± 0.156 |  16.021 |  16.328 |
| parse_large_chunks_simd_dummy     | 10.931 ± 0.044 |  10.896 |  10.995 |

## All implementations in single thread

| Command                           |       Mean [s] | Min [s] | Max [s] |    Relative |
|:----------------------------------|---------------:|--------:|--------:|------------:|
| naive_line_by_line                | 91.523 ± 0.482 |  90.955 |  92.110 | 5.43 ± 0.03 |
| naive_line_by_line_v2             | 58.189 ± 0.306 |  57.847 |  58.493 | 3.45 ± 0.02 |
| parse_large_chunks_as_bytes       | 33.714 ± 0.096 |  33.623 |  33.848 | 2.00 ± 0.01 |
| parse_large_chunks_as_i64         | 26.451 ± 0.478 |  26.122 |  27.162 | 1.57 ± 0.03 |
| parse_large_chunks_as_i64_v2      | 19.797 ± 0.031 |  19.764 |  19.830 | 1.18 ± 0.00 |
| parse_large_chunks_as_i64_unsafe  | 27.408 ± 0.390 |  27.058 |  27.780 | 1.63 ± 0.02 |
| parse_large_chunks_as_i64_as_java | 16.848 ± 0.050 |  16.784 |  16.900 |        1.00 |
| parse_large_chunks_simd           | 31.520 ± 0.082 |  31.437 |  31.632 | 1.87 ± 0.01 |
| parse_large_chunks_simd_v1        | 30.221 ± 0.339 |  29.961 |  30.689 | 1.79 ± 0.02 |

# Fastest Java solution [CalculateAverage_thomaswue.java](https://github.com/gunnarmorling/1brc/blob/main/src/main/java/dev/morling/onebrc/CalculateAverage_thomaswue.java)

| Type of run  | Number of threads |        Mean [s] | Min [s] | Max [s] |
|:-------------|-------------------|----------------:|--------:|--------:|
| JVM          | 1                 |   9.420 ± 0.060 |   9.327 |   9.504 |
| Native Image | 1                 |   8.647 ± 0.038 |   8.612 |   8.704 |
| JVM          | 48                |   2.133 ± 0.236 |   1.857 |   2.586 |
| Native Image | 48                | 0.4121 ± 0.0051 |  0.4019 |  0.4163 |

## JVM

```bash
hyperfine --warmup 4 --runs 10 --export-markdown java_thomaswue.md "java --enable-preview --class-path /root/code/github/gunnarmorling/1brc/target/average-1.0.0-SNAPSHOT.jar dev.morling.onebrc.CalculateAverage_thomaswue"
```

## Native Image

```bash
hyperfine --warmup 4 --runs 10 --export-markdown java_native_thomaswue.md /root/code/github/gunnarmorling/1brc/target/CalculateAverage_thomaswue_image
```