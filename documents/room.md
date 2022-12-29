# Room related messages

## Create room

### Request message

| Field name  | Type                                          | Required | Description                                                        | Default value |
|-------------|-----------------------------------------------|----------|--------------------------------------------------------------------|---------------|
| disconnect  | boolean                                       | N        | If user already joined to room, disconnect from it                 | false         |
| name        | string                                        | N        | Room name                                                          |               |
| access_type | [CreateRoomAccessType](#CreateRoomAccessType) | N        | Definition for who can access and see the room                     | Public        |
| max_user    | Number                                        | N        | Maximum number for participants. Use 0 for unlimited participants. | Unlimited     |

**Example requests**

```json
{
    "type": "Room",
    "room_type": "Create",
    "disconnect": true,
    "access_type": "Private",
    "max_user": 1
}
```

```json
{
    "type": "Room",
    "room_type": "Create",
    "tags": ["test 1", "test 2", "test 3"],
}
```



### Success Response

| Field name | Type    | Required | Description                 |
|------------|---------|----------|-----------------------------|
| status     | boolean | Y        | Value should be 'true'      |
| result     | string  | Y        | Room ID as uuid |

```json
{
    "status": true,
    "result": "4077a478-d01f-4c09-9462-c54f7ea0e2a7"
}
```

### Fail Response

| Field name | Type    | Required | Description                 |
|------------|---------|----------|-----------------------------|
| status     | boolean | Y        | Value should be 'false'     |
| result     | string  | Y        | Error message               |


**Example response**:

```json
{
    "status": false,
    "result": "User joined to other room"
}
```


## Message objects

### CreateRoomAccessType

Who can access the room.

| Field name    | Information |
|---------|-------|
| Public  | The room can be searchable by everyone and anyone can join to room.     |
| Private | The room available only with the key     |
| Friend  | Friends can see and can join to room     |


Except Tag field, all fields names are "Public", "Private" and "Friend".
