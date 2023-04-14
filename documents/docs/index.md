# Yummy Game Server

Yummy is the multiplayer game engine to make it easier to develop game. Supports websocket communication to give wider range of the platform. Web, Android, Ios, desktop and all modern browser have inbuild websocket support and **Yummy** can be used almost all platforms.
Our main goal of the developing this application is support game developer to make better games. Most of the time developing multiplayer games are more complicated than the single player games and with **Yummy** some of the difficulties can be solved more easier.

### Features
- Custom user metadata
- Custom rooms
- Different authentication methods
- Supports for vertical and horizontal scaling
- Observability via OpenTelemetry integration

### Todo list

- [ ] Close inactive connections
- [ ] Dispose inactive rooms
- [ ] Add parameter for OpenTelemetry configuration
- [ ] Design document
- [ ] Example projects
- [ ] Friend add/remove/list integration
- [ ] RabbitMQ integration
- [ ] Protocol Buffers or FlatBuffers integration
- [ ] Create Lua scripts for Redis operations
- [ ] Server disconnect detection [Stateless]
- [ ] Web interface for system control
- [ ] Client libraries (JS, Python, Rust, Godot, Unity, etc.)
- [ ] Create websocket tester application that control multiple connections
- [ ] Create **system** user at startup to configure system remotely
- [X] Support for pre and post API calls
- [X] Room metadata
- [X] Python runtimes
- [X] Multiple room support for user
- [X] Room join request
- [X] Ban and kick from room
- [X] Redis integration
- [X] TLS support
- [X] Remove room at redis state when no user in the room
- [X] Remove all unnecessary copy and clone

---

### General Unit Test

- [ ] Room integration
- [ ] Multiple server communication
- [ ] Integration test
