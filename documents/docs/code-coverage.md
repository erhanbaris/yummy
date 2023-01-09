Code coverage requires additional setup and has been tested on **Os X** system only. So there is no guarantee that it will work on linux or windows systems.

[grcov](https://github.com/mozilla/grcov) is used to do coverage and need to install.

!!! command "To generate code coverage result"
    ```bash
    RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-normal-%p-%m.profraw' cargo test --all

        grcov . \
            --binary-path ./target/debug/deps/ \
            --source-dir . \
            --ignore-not-existing \
            --excl-start 'mod test* \{' \
            --ignore 'tests/*' \
            --ignore server/src/main.rs \
            --ignore server/src/api/websocket/client.rs \
            --ignore general/src/client.rs \
            --ignore "*test.rs" \
            --ignore "*tests.rs" \
            -t html \
            -o html

    find . -name "*.profraw" -type f -delete
    ```

After executing commands, it will generate html folder and that include code coverage results. The current results shows that code coverage is more than **85%**.