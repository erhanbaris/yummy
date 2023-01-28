Code coverage requires additional setup and has been tested on **Os X** system only. So there is no guarantee that it will work on linux or windows systems.

[grcov](https://github.com/mozilla/grcov) is used to do coverage and need to install.

!!! command "To generate code coverage result"
    ```bash
    RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-normal-%p-%m.profraw' cargo test --all
    RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-stateless-%p-%m.profraw' cargo test --all --features stateless

    grcov . \
        --binary-path ./target/debug/deps/ \
        --source-dir . \
        --excl-start 'mod test* \{' \
        --ignore 'tests/*' \
        --ignore 'test/*' \
        --ignore server/src/main.rs \
        --ignore server/src/api/websocket/client.rs \
        --ignore general/src/client.rs \
        --ignore "*test.rs" \
        --ignore "*tests.rs" \
        --ignore "*github.com*" \
        --ignore "*libcore*" \
        --ignore "*rustc*" \
        --ignore "*liballoc*" \
        --ignore "*cargo*" \
        -t html \
        -o html

    find . -name "*.profraw" -type f -delete
    ```

After executing commands, it will generate html folder and that include code coverage results. The current results shows that code coverage is more than **85%**.