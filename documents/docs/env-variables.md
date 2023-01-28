# Environment variables

Yummy has multiple configuration over environtment variable. Those configurations can be passed via environtment or via config file. The config file must be located near the executable.

| Profile     | File name |
|-------------|-----------|
| Production  | :material-file-cog: prod.env  |
| Development | :material-file-cog: dev.env   |
| Test        | :material-file-cog: test.env  |

## Parameters


### `SERVER_NAME` <br/>
The system generates a random name at the startup. The name will start with YUMMY and continue with 7 alphanumeric characters. This name will be used to communicate between Yummy instances and shorter names can improve performance. Example names: YUMMY-wvO8T0u, YUMMY-NTXBzdo, YUMMY-oSArCvq. <br/>
:octicons-milestone-24: **Default**: `Server name randomly generated.` <br/>

### `BIND_IP` <br/>
Instance's binding ip address. <br/>
:octicons-milestone-24: **Default**: `0.0.0.0` <br/>

### `BIND_PORT` <br/>
Instance's binding port address. <br/>
:octicons-milestone-24: **Default**: `9090` <br/>

### `TLS_CERT_PATH` <br/>
TLS certificates cert file path. <br/>
:octicons-milestone-24: **Default**: ` ` <br/>

### `TLS_KEY_PATH` <br/>
TLS certificates key file path. <br/>
:octicons-milestone-24: **Default**: ` ` <br/>

### `RUST_LOG` <br/>
Rust Programming Language's and Actix Framework's logging configuration. <br/>
[Rust logging information](https://docs.rs/env_logger/0.10.0/env_logger/#enabling-logging) <br/>
:octicons-milestone-24: **Default**: `debug,backend,actix_web=debug` <br/>

### `HEARTBEAT_INTERVAL` <br/>
Heartbeat message sent interval. This parameter is in **seconds**. <br/>
:octicons-milestone-24: **Default**: `10` <br/>

### `HEARTBEAT_TIMEOUT` <br/>
Maximum wait time after receiving the last heartbeat message. The connection termination procedure will be started if the system cannot receive a heartbeat message in time. This parameter is in **seconds**. <br/>
:octicons-milestone-24: **Default**: `20` <br/>

### `CONNECTION_RESTORE_WAIT_TIMEOUT` <br/>
If the client disconnected from the instance, the system wait some amound of time to informatim other users and update clients states. This time starts at after user disconnect from the instance or hit to **HEARTBEAT_TIMEOUT**. This parameter is in **seconds**. <br/>
:octicons-milestone-24: **Default**: `10` <br/>

### `TOKEN_LIFETIME` <br/>
JWT lifetime. That parameter used for session and connection restoration. This parameter is in **milliseconds**. <br/>
:octicons-milestone-24: **Default**: `86400` <br/>

### `API_KEY_NAME` <br/>
Websocket's HTTP GET parameter name. <br/>
:octicons-milestone-24: **Default**: `x-yummy-api` <br/>

### `INTEGRATION_KEY` <br/>
Websocket's integration key to communicate. <br/>
:octicons-milestone-24: **Default**: `YummyYummy` <br/>

### `SALT_KEY` <br/>
Secret key for JWT. <br/>
:octicons-milestone-24: **Default**: `YUMMY-SALT` <br/>

### `MAX_USER_META` <br/>
Maximum allowed meta informations per users. <br/>
:octicons-milestone-24: **Default**: `10` <br/>

### `DEFAULT_MAX_ROOM_META` <br/>
Maximum allowed meta informations per room. <br/>
:octicons-milestone-24: **Default**: `10` <br/>

### `ROOM_PASSWORD_CHARSET` <br/>
Automatic generated room password's charset. <br/>
:octicons-milestone-24: **Default**: `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789` <br/>

### `ROOM_PASSWORD_LENGTH` <br/>
Automatic generated room password's length. <br/>
:octicons-milestone-24: **Default**: `4` <br/>

### `DATABASE_PATH` <br/>
Sqlite database path. <br/>
:octicons-milestone-24: **Default**: `yummy.db` <br/>

### `DEFAULT_LUA_FILES_PATH` <br/>
Lua script files location. <br/>
:octicons-milestone-24: **Default**: `./server/lua/` <br/>

### `REDIS_URL` <br/>
Redis connection information. <br/>
:octicons-milestone-24: **Default**: `redis://127.0.0.1/` <br/>

### `REDIS_PREFIX` <br/>
Prefix for all Redis keys. <br/>
:octicons-milestone-24: **Default**: ` ` <br/>
