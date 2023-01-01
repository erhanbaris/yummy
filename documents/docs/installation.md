Yummy Game Engine is developed with the Rust Programming Language. The system save all user, room and informations locally at Sqlite database. Also, the Redis used for caching informations based on build parameters.

If you want to start Yummy application locally please execute the following command:

!!! command "To start Yummy application"
    ```bash
    cargo run --release
    ```

The application has multiple parameters to configure. All these configuration parameters can be found in the [link](env-variables.md).

Also, there are `stateless` definition to use Redis server as a cache manager.


!!! command "To start Yummy application with stateless mode"
    ```bash
    cargo run --release --features stateless
    ```

When Yummy starts, you will see messages like this on the console.

```bash
2023-01-01T17:30:35.039755Z  INFO server: Yummy is starting...    
2023-01-01T17:30:35.039809Z  INFO server: Binding at   "0.0.0.0:9090"    
2023-01-01T17:30:35.039824Z  INFO server: Server name  "YUMMY-BRaNf5T"    
2023-01-01T17:30:35.039841Z  INFO server: Log level is "debug,backend,actix_web=debug"    
2023-01-01T17:30:35.042950Z  INFO actix_server::builder: Starting 4 workers
2023-01-01T17:30:35.043081Z  INFO actix_server::server: Actix runtime found; starting in Actix runtime
```

For now, we don't have a support Docker yet. It will come with future releases.