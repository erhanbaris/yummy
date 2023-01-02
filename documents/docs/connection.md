Yummy is designed to communicate over WebSocket. Almost all modern programming languages has a Websocket support so it should not be a problem to connect. Also, all modern browsers has a Websocket support and that makes Yummy to accesible from browsers.

!!! url "URI syntax for without TLS"
    ```
    ws://127.0.0.1:9090/v1/socket?x-yummy-api=YummyYummy
    ```

!!! url "URI syntax for with TLS"
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

## Example usages
!!! example "Basic authentication example (Javascript)"
    ```javascript linenums="1" hl_lines="5 7-13"
    if ("WebSocket" in window) {
        alert("WebSocket is supported by your Browser!");

        // Let us open a web socket
        var ws = new WebSocket("ws://127.0.0.1:9090/v1/socket?x-yummy-api=YummyYummy");
        ws.onopen = function() {
            ws.send(JSON.stringify({
                "type": "Auth",
                "auth_type": "Email",
                "email": "test@test.com",
                "password": "test",
                "create": true
            }));
        };

        ws.onmessage = function(evt) {
            alert(evt.data);
        };

        ws.onclose = function() {
            alert("Connection is closed...");
        };
    } else {
        alert("WebSocket NOT supported by your Browser!");
    }
    ```