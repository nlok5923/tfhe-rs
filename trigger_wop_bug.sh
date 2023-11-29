#!/usr/bin/env bash

failed=0
for i in {1..1000}; do
    echo "run $i"
    # uncomment below to force the generation of a failing key, then comment and investigate if a set
    # of inputs makes the test fail repeatedly
    # rm -rf keys
    RUSTFLAGS="-C target-cpu=native" cargo +stable nextest \
    run --tests --cargo-profile release --package tfhe --profile ci \
    --features=x86_64-unix,shortint,internal-keycache --test-threads 128 \
    -E "test(shortint::wopbs::test::test_programmable_bootstrapping_native_crt_doctest_ci_run_filter)"
    if [[ "$?" != "0" ]]; then
        ((failed=failed+1))
        echo "${failed} fails so far"
    fi
done

echo "${failed} fails in 1000 runs"
