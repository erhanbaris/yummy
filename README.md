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

### Features
- Custom user metadata
- Custom rooms
- Different authentication methods
- Supports for vertical and horizontal scaling
- Observability via OpenTelemetry integration

### Todo list

- [ ] Add parameter for OpenTelemetry configuration
- [X] TLS support
- [ ] Remove room at redis state when no user in the room
- [ ] Design document
- [ ] Example projects
- [ ] Friend add/remove/list integration
- [ ] RabbitMQ integration
- [X] Redis integration
- [ ] Create Lua scripts for Redis operations
- [ ] Server disconnect detection [Stateless]
- [ ] Support for pre and post API calls
- [ ] Web interface for system control
- [ ] Lua, JS, .NET Core runtimes
- [ ] Client libraries (JS, Python, Rust, Godot, Unity, etc.)

---

### General Unit Test

- [ ] Room integration
- [ ] Multiple server communication
- [ ] Integration test


## Coverage report generation procedures

RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --all

grcov . --binary-path ./target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o html

find . -name "*.profraw" -type f -delete