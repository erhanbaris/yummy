# Yummy Game Server
[![Rust](https://github.com/erhanbaris/yummy/actions/workflows/rust.yml/badge.svg)](https://github.com/erhanbaris/yummy/actions/workflows/rust.yml)

Yummy is the multiplayer game engine to make it easier to develop game. Supports websocket communication to give wider range of the platform. Web, Android, Ios, desktop and all modern browser have inbuild websocket support and **Yummy** can be used almost all platforms.
Our main goal of the developing this application is support game developer to make better games. Most of the time developing multiplayer games are more complicated than the single player games and with **Yummy** some of the difficulties can be solved more easier.

## Installation
Requires Rust Language to build the Yummy.
```bash
cargo run --release
```

## Unit test executions 

To execute all unit test, need to execute following commands. The second command require Redis instance.

```bash
cargo test --all
cargo test --all  --features stateless
```

## Documentation
[Link](documents/README.md)

### Features
- Custom user metadata
- Custom rooms
- Different authentication methods
- Supports for vertical and horizontal scaling
- Observability via OpenTelemetry integration
