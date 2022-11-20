# Yummy Game Server
[![Rust](https://github.com/erhanbaris/yummy/actions/workflows/rust.yml/badge.svg)](https://github.com/erhanbaris/yummy/actions/workflows/rust.yml)


## Coverage report generation procedures

RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --all

grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o html

find . -name "*.profraw" -type f -delete
