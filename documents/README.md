# Environment arguments

Yummy has multiple configuration over environtment variable. Those configurations can be passed via environtment or via config file. The config file must be located near the executable.

| Profile     | File name |
|-------------|-----------|
| Production  | prod.env  |
| Development | dev.env   |
| Test        | test.env  |

### Parameters
**SERVER_NAME**
__Default value__: Server name randomly generated.
__Information__: The system generates a random name at the startup. The name will start with YUMMY and continue with 7 alphanumeric characters. This name will be used to communicate between Yummy instances and shorter names can improve performance. Example names: YUMMY-wvO8T0u, YUMMY-NTXBzdo, YUMMY-oSArCvq.

**BIND_IP**
__Default value__: 0.0.0.0
__Information__: Instance's binding ip address.

**BIND_PORT**
__Default value__: 9090
__Information__: Instance's binding port address.

**RUST_LOG**
__Default value__: debug,backend,actix_web=debug
__Information__: Rust Programming Language's and Actix Framework's logging configuration.
[Rust logging information](https://docs.rs/env_logger/0.10.0/env_logger/#enabling-logging)

**HEARTBEAT_INTERVAL**
__Default value__: 10
__Information__: Heartbeat message sent interval. This parameter is in **seconds**.

**HEARTBEAT_TIMEOUT**
__Default value__: 20
__Information__: Maximum wait time after receiving the last heartbeat message. The connection termination procedure will be started if the system cannot receive a heartbeat message in time. This parameter is in **seconds**.

**CONNECTION_RESTORE_WAIT_TIMEOUT**
__Default value__: 10
__Information__: If the client disconnected from the instance, the system wait some amound of time to informatim other users and update clients states. This time starts at after user disconnect from the instance or hit to **HEARTBEAT_TIMEOUT**. This parameter is in **seconds**.

**TOKEN_LIFETIME**
__Default value__: 86400
__Information__: JWT lifetime. That parameter used for session and connection restoration. This parameter is in **milliseconds**.

**API_KEY_NAME**
__Default value__: x-yummy-api
__Information__: Websocket's HTTP GET parameter name.

**SALT_KEY**
__Default value__: YUMMY-SALT
__Information__: Secret key for JWT.

**MAX_USER_META**
__Default value__: 10
__Information__: Maximum allowed meta informations per users.

**ROOM_PASSWORD_CHARSET**
__Default value__: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789
__Information__: Automatic generated room password's charset.

**ROOM_PASSWORD_LENGTH**
__Default value__: 4
__Information__: Automatic generated room password's length.

**DATABASE_PATH**
__Default value__: yummy.db
__Information__: Sqlite database path.

**REDIS_URL**
__Default value__: redis://127.0.0.1/
__Information__: Redis connection information.

**REDIS_PREFIX**
__Default value__: 
__Information__: Prefix for all Redis keys.


# Communication messages

[Authentication messages](auth.md)

[User related messages](user.md)

[Room related messages](room.md)
