# Environment arguments

Yummy has multiple configuration over environtment variable. Those configurations can be passed via environtment or via config file. The config file must be located near the executable.

| Profile     | File name |
|-------------|-----------|
| Production  | prod.env  |
| Development | dev.env   |
| Test        | test.env  |

### Parameters 
* **SERVER_NAME** <br/>
__Default value__: Server name randomly generated. <br/>
__Information__: The system generates a random name at the startup. The name will start with YUMMY and continue with 7 alphanumeric characters. This name will be used to communicate between Yummy instances and shorter names can improve performance. Example names: YUMMY-wvO8T0u, YUMMY-NTXBzdo, YUMMY-oSArCvq. <br/>

* **BIND_IP** <br/>
__Default value__: 0.0.0.0 <br/>
__Information__: Instance's binding ip address. <br/>

* **BIND_PORT** <br/>
__Default value__: 9090 <br/>
__Information__: Instance's binding port address. <br/>

* **TLS_CERT_PATH** <br/>
__Default value__: <br/>
__Information__: TLS certificates cert file path. <br/>

* **TLS_KEY_PATH** <br/>
__Default value__: <br/>
__Information__: TLS certificates key file path. <br/>

* **RUST_LOG** <br/>
__Default value__: debug,backend,actix_web=debug <br/>
__Information__: Rust Programming Language's and Actix Framework's logging configuration. <br/>
[Rust logging information](https://docs.rs/env_logger/0.10.0/env_logger/#enabling-logging) <br/>

* **HEARTBEAT_INTERVAL** <br/>
__Default value__: 10 <br/>
__Information__: Heartbeat message sent interval. This parameter is in **seconds**. <br/>

* **HEARTBEAT_TIMEOUT** <br/>
__Default value__: 20 <br/>
__Information__: Maximum wait time after receiving the last heartbeat message. The connection termination procedure will be started if the system cannot receive a heartbeat message in time. This parameter is in **seconds**. <br/>

* **CONNECTION_RESTORE_WAIT_TIMEOUT** <br/>
__Default value__: 10 <br/>
__Information__: If the client disconnected from the instance, the system wait some amound of time to informatim other users and update clients states. This time starts at after user disconnect from the instance or hit to **HEARTBEAT_TIMEOUT**. This parameter is in **seconds**. <br/>

* **TOKEN_LIFETIME** <br/>
__Default value__: 86400 <br/>
__Information__: JWT lifetime. That parameter used for session and connection restoration. This parameter is in **milliseconds**. <br/>

* **API_KEY_NAME** <br/>
__Default value__: x-yummy-api <br/>
__Information__: Websocket's HTTP GET parameter name. <br/>

* **SALT_KEY** <br/>
__Default value__: YUMMY-SALT <br/>
__Information__: Secret key for JWT. <br/>

* **MAX_USER_META** <br/>
__Default value__: 10 <br/>
__Information__: Maximum allowed meta informations per user. <br/>

* **MAX_ROOM_META** <br/>
__Default value__: 10 <br/>
__Information__: Maximum allowed meta informations per room. <br/>

* **ROOM_PASSWORD_CHARSET** <br/>
__Default value__: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 <br/>
__Information__: Automatic generated room password's charset. <br/>

* **ROOM_PASSWORD_LENGTH** <br/>
__Default value__: 4 <br/>
__Information__: Automatic generated room password's length. <br/>

* **DATABASE_PATH** <br/>
__Default value__: yummy.db <br/>
__Information__: Sqlite database path. <br/>

* **REDIS_URL** <br/>
__Default value__: redis://127.0.0.1/ <br/>
__Information__: Redis connection information. <br/>

* **REDIS_PREFIX** <br/>
__Default value__: <br/>
__Information__: Prefix for all Redis keys. <br/>


# Communication messages

[Authentication messages](auth.md)

[User related messages](user.md)

[Room related messages](room.md)
