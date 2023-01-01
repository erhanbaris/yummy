Yummy is designed to communicate over WebSocket. Almost all modern programming languages has a Websocket support so it should not be a problem to connect. Also, all modern browsers has a Websocket support and that makes Yummy to accesible from browsers.

!!! abstract "URI syntax for without TLS"
    ```
    ws://127.0.0.1:9090/v1/socket?x-yummy-api=YummyYummy
    ```

!!! abstract "URI syntax for with TLS"
    ```
    wss://127.0.0.1:9090/v1/socket?x-yummy-api=YummyYummy
    ```

Please check configuration parameters to personalize it.


`BIND_IP` <br/>

`BIND_PORT` <br/>

`API_KEY_NAME` <br/>

`INTEGRATION_KEY` <br/>


[:material-file-settings: Environtment variables](env-variables.md)


Yummy will not sent any message when the client connected. The system wait message from client and give response as a result.
