# Room related messages

## Create room

=== ":inbox_tray: Request message"
    !!! success ""
        | Field name     | Type                              | Required | Description                                                                                                                              |
        |----------------|-----------------------------------|----------|------------------------------------------------------------------------------------------------------------------------------------------|
        | `type`         | string                            | Y        | Value must be **CreateRoom**                                                                                                                |
        | `join_request` | boolean                           | N        | User need an approvement from moderator or admin to join the room. Default: **false**                                                    |
        | `name`         | string                            | N        | Room name                                                                                                                                |
        | `description`  | string                            | N        | Room description                                                                                                                         |
        | `access_type`  | [AccessType](#accesstype)         | N        | Definition for who can access and see the room. Default: **0**                                                                           |
        | `max_user`     | number                            | N        | Maximum number for participants. Use 0 for unlimited participants. Default:  **0**                                                       |
        | `tags`         | [string]                          | N        | Array of tag.                                                                                                                            |
        | `metas`        | [[Meta]](general-objects.md#meta) | N        | Array of [Meta](general-objects.md#meta) information. This is room based information and have access level to whom see that information. |

        **Example requests:**

        === "Example 1"
            ```json
            {
                "type": "CreateRoom",
                "access_type": 1,
                "max_user": 1,
                "metas": {
                    "min-score": 5000,
                    "country": "DK"
                }
            }
            ```
        === "Example 2"
            ```json
            {
                "type": "CreateRoom",
                "tags": ["test 1", "test 2", "test 3"]
            }
            ```


=== ":outbox_tray: Response message"
    !!! success ""
        === ":material-check: Success"
            | Field name | Type    | Nullable | Description                   |
            |------------|---------|----------|-------------------------------|
            | `status`   | boolean | N        | Value should be **true**      |
            | `type`     | string  | N        | Value must be **RoomCreated** |
            | `room_id`  | string  | N        | Room's ID                     |

            **Example requests:**

            ```json
            {
                "status": true,
                "type": "CreateRoom",
                "room_id": "8e4d7516-1ee7-47d2-9387-438de3db37b9"
            }
            ```
        === ":octicons-x-16: Fail"

            | Field name | Type    | Nullable | Description                 |
            |------------|---------|----------|-----------------------------|
            | `status`     | boolean | N        | Value should be **false** |
            | `error`      | string  | N        | Error message             |


            **Example response:**:

            ```json
            {
                "status": false,
                "error": "User joined to other room"
            }
            ```

## Join to room
Joining to room require a little more attention than other parts. Room can be configurable based on the owners decitions. It means that, there are couple of parameters and based on parameters, user can be join to room directly or wait in the lobby to be accepted by room owner or moderator.


=== ":inbox_tray: Request message"
    !!! success ""
        | Field name       | Type                          | Required | Description                  | Default value |
        |------------------|-------------------------------|----------|------------------------------|---------------|
        | `type`           | string                        | Y        | Value must be **JoinToRoom** |               |
        | `room_id`        | string                        | Y        | Room's ID                    |               |
        | `room_user_type` | [RoomUserType](#roomusertype) | N        | User type at the room        | 1             |

        **Example requests:**

        === "Example 1"
            ```json
            {
                "type": "JoinToRoom",
                "room_id": "8c366421-f7d8-47e1-8eed-82915280ce30"
            }
            ```
        === "Example 2"
            ```json
            {
                "type": "JoinToRoom",
                "room_id": "8c366421-f7d8-47e1-8eed-82915280ce30",
                "room_user_type": 3
            }
            ```


=== ":outbox_tray: Response message"
    !!! success ""
        === ":material-check: Success"
            | Field name  | Type                    | Nullable | Description                                                                                                                    |
            |-------------|-------------------------|----------|--------------------------------------------------------------------------------------------------------------------------------|
            | `status`    | boolean                 | N        | Value should be **true**                                                                                                       |
            | `type`      | string                  | N        | Value must be **Joined**                                                                                                       |
            | `room_id`   | string                  | N        | Room's ID                                                                                                                      |
            | `room_name` | string                  | Y        | Room's name                                                                                                                    |
            | `users`     | [RoomUser](#roomuser)   | N        | Array of [RoomUser](#roomuser).                                                                                                |
            | `metas`     | [[Meta]](general-objects.md#meta) | N        | Array of [Meta](general-objects.md#meta) information. This is room based information and have access level to whom see that information. |
            
            **Example requests:**

            ```json
            {
                "status": true,
                "type": "JoinToRoom",
                "room_name": null,
                "users": [
                    {
                        "user_id": "bf66435f-705a-48aa-aeed-da06e5e29833",
                        "name": null,
                        "type": 1
                    },
                    {
                        "user_id": "8c365226-06cd-4140-9e31-f6b9a73d6b78",
                        "name": null,
                        "type": 3
                    }
                ]
            }
            ```
        === ":octicons-x-16: Fail"

            | Field name | Type    | Nullable | Description                 |
            |------------|---------|----------|-----------------------------|
            | `status`     | boolean | N        | Value should be **false** |
            | `error`      | string  | N        | Error message             |


            **Example response:**:

            ```json
            {
                "status": false,
                "error": "User joined to other room"
            }
            ```


# Message objects

### :material-table: AccessType

Who can access the room.

| Value | Meaning | Information                                                         |
|:-----:|---------|---------------------------------------------------------------------|
| `0`   | Public  | The room can be searchable by everyone and anyone can join to room. |
| `1`   | Private | The room available only with the key                                |
| `2`   | Friend  | Friends can see and can join to room                                |

### :material-table: RoomUserType

User's authorization in this room

| Value | Meaning   |
|-------|-----------|
| `1`   | User      |
| `2`   | Moderator |
| `3`   | Owner     |
