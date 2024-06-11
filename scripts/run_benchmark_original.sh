#!/usr/bin/env bash

set -eux

cur_dir=$(pwd)
patch_path="${cur_dir}/scripts/customize_original.patch"
echo $patch_path

cd /tmp
git clone --depth=1 https://github.com/tumdum/1brc.git && cd 1brc && git checkout 34c761a9fa729fc121d01ef5abd25d16141f28e4
git apply "${cur_dir}/scripts/customize_original.patch"

rustflags="-C target-cpu=native" cargo build --release

run_app="./target/release/rs /media/ramdisk/measurements.txt"

function run_benchmark() {
    hyperfine --warmup 1 --runs 4 --export-markdown "${1}_threads.md" "${run_app} $1"
}

run_benchmark 1
run_benchmark 2
run_benchmark 4
run_benchmark 8
run_benchmark 16
run_benchmark 24
run_benchmark 32
run_benchmark 48