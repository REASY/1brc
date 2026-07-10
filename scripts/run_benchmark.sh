#!/usr/bin/env bash

set -euo pipefail

usage() {
    echo "Usage: $0 <measurements-file> [results-directory]" >&2
    echo "" >&2
    echo "Environment variables:" >&2
    echo "  HYPERFINE_WARMUP  Warmup runs per command (default: 1)" >&2
    echo "  HYPERFINE_RUNS    Measured runs per command (default: 4)" >&2
    echo "  THREAD_COUNTS     Space-separated scaling thread counts" >&2
    echo "                    (default: 1 2 4 8 16 24 32)" >&2
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
    usage
    exit 2
fi

if [[ ! -f "$1" ]]; then
    echo "Measurements file does not exist: $1" >&2
    exit 2
fi

measurements_file=$(realpath "$1")
results_dir=${2:-benchmark-results}
warmup=${HYPERFINE_WARMUP:-1}
runs=${HYPERFINE_RUNS:-4}
read -r -a thread_counts <<< "${THREAD_COUNTS:-1 2 4 8 16 24 32}"

if ! command -v hyperfine >/dev/null 2>&1; then
    echo "hyperfine is required but was not found in PATH" >&2
    exit 127
fi

mkdir -p "$results_dir"
cargo build --release

printf -v quoted_measurements_file '%q' "$measurements_file"
run_app="./target/release/brc-app ${quoted_measurements_file}"

dummy_implementations=(
    naive_line_by_line_dummy
    parse_large_chunks_as_bytes_dummy
    parse_large_chunks_as_i64_dummy
    parse_large_chunks_simd_dummy
)

implementations=(
    naive_line_by_line
    naive_line_by_line_v2
    parse_large_chunks_as_bytes
    parse_large_chunks_as_i64
    parse_large_chunks_as_i64_v2
    parse_large_chunks_as_i64_unsafe
    parse_large_chunks_as_i64_as_java
    parse_large_chunks_simd
    parse_large_chunks_simd_v1
    parse_large_chunks_simd_v2
    parse_large_chunks_memchr_table
    parse_large_chunks_std_simd_table
    parse_large_chunks_simd_temperature_table
    parse_large_chunks_full_simd_table
)

# Keep scaling runs focused on the previous scaling baselines and all new
# contenders. Every implementation is still measured in the single-thread run.
scaling_implementations=(
    parse_large_chunks_as_i64_v2
    parse_large_chunks_as_i64_as_java
    parse_large_chunks_memchr_table
    parse_large_chunks_std_simd_table
    parse_large_chunks_simd_temperature_table
    parse_large_chunks_full_simd_table
)

run_hyperfine() {
    local output_file=$1
    local threads=$2
    shift 2

    local commands=()
    local implementation
    for implementation in "$@"; do
        commands+=("${run_app} ${threads} ${implementation}")
    done

    hyperfine \
        --warmup "$warmup" \
        --runs "$runs" \
        --export-markdown "$output_file" \
        "${commands[@]}"
}

run_hyperfine "$results_dir/dummy.md" 1 "${dummy_implementations[@]}"
run_hyperfine "$results_dir/1_threads.md" 1 "${implementations[@]}"

for threads in "${thread_counts[@]}"; do
    if [[ "$threads" == 1 ]]; then
        continue
    fi
    run_hyperfine \
        "$results_dir/${threads}_threads.md" \
        "$threads" \
        "${scaling_implementations[@]}"
done
