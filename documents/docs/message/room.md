# Room related messages

## Create room

### Request message

!!! success ""
    | Field name    | Type                                          | Required | Description                                                        | Default value |
    |---------------|-----------------------------------------------|----------|--------------------------------------------------------------------|---------------|
    | `type`        | string                                        | Y        | Value must be **Room**                                             |               |
    | `room_type`   | string                                        | Y        | Value must be **Create**                                           |               |
    | `disconnect`  | boolean                                       | N        | If user already joined to room, disconnect from it                 | false         |
    | `name`        | string                                        | N        | Room name                                                          |               |
    | `access_type` | [CreateRoomAccessType](#createroomaccesstype) | N        | Definition for who can access and see the room                     | 0             |
    | `max_user`    | number                                        | N        | Maximum number for participants. Use 0 for unlimited participants. | 0             |

    **Example requests:**

    === "Example 1"
        ```json
        {
            "type": "Room",
            "room_type": "Create",
            "disconnect": true,
            "access_type": 1,
            "max_user": 1
        }
        ```
    === "Example 2"
        ```json
        {
            "type": "Room",
            "room_type": "Create",
            "tags": ["test 1", "test 2", "test 3"],
        }
        ```


### Response

!!! success ""
    === "Success"
        | Field name | Type    | Required | Description                 |
        |------------|---------|----------|-----------------------------|
        | `status`     | boolean | Y        | Value should be 'true'      |
        | `result`     | string  | Y        | Room ID as uuid |

        **Example requests:**

        ```json
        {
            "status": true,
            "result": "4077a478-d01f-4c09-9462-c54f7ea0e2a7"
        }
        ```
    === "Fail"

        | Field name | Type    | Required | Description                 |
        |------------|---------|----------|-----------------------------|
        | `status`     | boolean | Y        | Value should be 'false'   |
        | `result`     | string  | Y        | Error message             |


        **Example response:**:

        ```json
        {
            "status": false,
            "result": "User joined to other room"
        }
        ```


# Message objects

### :material-table: CreateRoomAccessType

Who can access the room.

| Value | Meaning | Information                                                         |
|:-----:|---------|---------------------------------------------------------------------|
| `0`   | Public  | The room can be searchable by everyone and anyone can join to room. |
| `1`   | Private | The room available only with the key                                |
| `2`   | Friend  | Friends can see and can join to room                                |

