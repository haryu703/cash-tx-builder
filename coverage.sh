#!/bin/bash

set -e

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"

rustup run nightly cargo build --all-features
rustup run nightly cargo test --all-features

zip -0 ccov.zip `find . \( -name "cash_tx_builder*.gc*" \) -print`
grcov ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore-dir "/*" > lcov.info
genhtml -o report/ --show-details --highlight --ignore-errors source --legend lcov.info