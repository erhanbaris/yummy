# Yummy Game Server
[![Rust](https://github.com/erhanbaris/yummy/actions/workflows/rust.yml/badge.svg)](https://github.com/erhanbaris/yummy/actions/workflows/rust.yml)

Yummy is the multiplayer game engine to make it easier to develop game. Supports websocket communication to give wider range of the platform. Web, Android, Ios, desktop and all modern browser have inbuild websocket support and **Yummy** can be used almost all platforms.
Our main goal of the developing this application is support game developer to make better games. Most of the time developing multiplayer games are more complicated than the single player games and with **Yummy** some of the difficulties can be solved more easier.

## Installation
Requires Rust Language to build the Yummy.
```bash
cargo run --release
```

## Documentation
[Link](documents/README.md)

### Todo list

- [ ] Friend add/remove/list integration
- [ ] RabbitMQ integration
- [X] Redis integration
- [ ] Server disconnect detection [Stateless]

---

### General Unit Test

- [ ] Room integration

---

### Stateless Unit Test

- [ ] Multiple server communication


## Coverage report generation procedures

RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --all

grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o html

find . -name "*.profraw" -type f -delete