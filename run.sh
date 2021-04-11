#!/usr/bin/env bash

set -Cex

# Logging level: 'error' | 'warn' | 'info' | 'debug' | 'trace'
case "${1}" in
    info)
        export CARGO_LOG_LEVEL="--verbose"
        ;;
    debug)
        export RUST_BACKTRACE=1
        export CARGO_LOG_LEVEL="--verbose"
        ;;
    trace)
        export RUST_BACKTRACE="full"
        export CARGO_LOG_LEVEL="--verbose"
        ;;
esac

# Target build: 'debug' | 'release'
case "${2}" in
    release)
        export RELEASE_FLAG="--release"
        ;;
esac

cargo fmt --all
cargo clippy
cargo test
cargo run ${CARGO_LOG_LEVEL} ${RELEASE_FLAG} -- "${3}"
