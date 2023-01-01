Code coverage requires additional setup and has been tested on **Os X** system only. So there is no guarantee that it will work on linux or windows systems.

[grcov](https://github.com/mozilla/grcov) is used to do coverage and need to install.

```sh
RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --all

grcov . \
    --binary-path ./target/debug/deps/ \
    -s . \
    -t html \
    --ignore-not-existing \
    --ignore '../*' \
    --ignore "/*" \
    --ignore server/src/main.rs \
    --ignore server/src/api/websocket/client.rs \
    --ignore "*test.rs" \
    --ignore "*tests.rs" \
    -o html

find . -name "*.profraw" -type f -delete
```

After executing commands, it will generate html folder and that include code coverage results. The current results shows that code coverage is more than **85%**.