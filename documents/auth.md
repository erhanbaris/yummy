# Authentication related messages

## Authenticate via email

### Request message

| Field name | Type    | Required | Description                                    | Default value |
|------------|---------|----------|------------------------------------------------|---------------|
| type       | string  | Y        | Value must be 'Auth'                          |               |
| auth_type  | string  | Y        | Value must be 'Email'                          |               |
| email      | string  | Y        | Authentication email address                   |               |
| password   | string  | Y        | Authentication password                        |               |
| create     | boolean | N        | If the user is not created yet, create new one | false         |

**Example request**

```json
{
    "type": "Auth",
    "auth_type": "Email",
    "email": "erhanbaris@gmail.com",
    "password": "erhan",
    "create": true
}
```

[Success response](#success-response)

[Fail response](#fail-response)


## Authenticate via device id

### Request message

| Field name | Type    | Required | Description                                    |
|------------|---------|----------|------------------------------------------------|
| type       | string  | Y        | Value must be 'Auth'                          |
| auth_type  | string  | Y        | Value must be 'DeviceId'                       |
| id         | string  | Y        | Authentication device id                       |

**Example request**

```json
{
    "type": "Auth",
    "auth_type": "DeviceId",
    "id": "1234567890"
}
```

## Authenticate via custom id

### Request message

| Field name | Type    | Required | Description                                    |
|------------|---------|----------|------------------------------------------------|
| type       | string  | Y        | Value must be 'Auth'                          |
| auth_type  | string  | Y        | Value must be 'CustomId'                       |
| id         | string  | Y        | Authentication device id                       |

**Example request**

```json
{
    "type": "Auth",
    "auth_type": "CustomId",
    "id": "ABV123"
}
```


## Refreshing token
Regenerating token with new expire date.

### Request message

| Field name | Type    | Required | Description                                    |
|------------|---------|----------|------------------------------------------------|
| type       | string  | Y        | Value must be 'Auth'                          |
| auth_type  | string  | Y        | Value must be 'Refresh'                        |
| token      | string  | Y        | Valid Authentication token                     |

**Example request**

```json
{
    "type": "Auth",
    "auth_type": "Refresh",
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2NzA1Mjg3MTEsInVzZXIiOnsiaWQiOiJiMDhkN2I3OS0xNDA1LTQxZGMtODJhMS02YTg4MjU3OWM3MmEiLCJzZXNzaW9uIjoiM2IxMzdjYWUtZmY1OC00NjY5LTg1YjctOWEyM2NiOGRiYzAxIiwibmFtZSI6bnVsbCwiZW1haWwiOiJlcmhhbmJhcmlzQGdtYWlsLmNvbSJ9fQ.6tLnsjWPRCz0cW00j2nzV-SUk6GwrlYgxe9V_p5mhxU"
}
```

[Success response](#success-response)

[Fail response](#fail-response)


## Restoring session
The user should be restore token after reconnecting to the system. If timeout exceeded, the session will be terminated and active game and player will be informed.

### Request message

| Field name | Type    | Required | Description                                    |
|------------|---------|----------|------------------------------------------------|
| type       | string  | Y        | Value must be 'Auth'                          |
| auth_type  | string  | Y        | Value must be 'Restore'                        |
| token      | string  | Y        | Valid Authentication token                     |

**Example request**

```json
{
    "type": "Auth",
    "auth_type": "Restore",
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2NzA1Mjg3MTEsInVzZXIiOnsiaWQiOiJiMDhkN2I3OS0xNDA1LTQxZGMtODJhMS02YTg4MjU3OWM3MmEiLCJzZXNzaW9uIjoiM2IxMzdjYWUtZmY1OC00NjY5LTg1YjctOWEyM2NiOGRiYzAxIiwibmFtZSI6bnVsbCwiZW1haWwiOiJlcmhhbmJhcmlzQGdtYWlsLmNvbSJ9fQ.6tLnsjWPRCz0cW00j2nzV-SUk6GwrlYgxe9V_p5mhxU"
}
```

[Success response](#success-response)

[Fail response](#fail-response)



## Logout
Terminates the current session and active game and player will be informed.

### Request message

| Field name | Type    | Required | Description                                    |
|------------|---------|----------|------------------------------------------------|
| type       | string  | Y        | Value must be 'Auth'                          |
| auth_type  | string  | Y        | Value must be 'Logout'                        |

**Example request**

```json
{
    "type": "Auth",
    "auth_type": "Logout"
}
```

[Success response](#success-response)

[Fail response](#fail-response)


### Success Response

| Field name | Type    | Required | Description                 |
|------------|---------|----------|-----------------------------|
| status     | boolean | Y        | Value should be 'true'      |
| result     | string  | Y        | User's authentication token |

```json
{
    "status": true,
    "result": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2NzA1Mjg3MTEsInVzZXIiOnsiaWQiOiJiMDhkN2I3OS0xNDA1LTQxZGMtODJhMS02YTg4MjU3OWM3MmEiLCJzZXNzaW9uIjoiM2IxMzdjYWUtZmY1OC00NjY5LTg1YjctOWEyM2NiOGRiYzAxIiwibmFtZSI6bnVsbCwiZW1haWwiOiJlcmhhbmJhcmlzQGdtYWlsLmNvbSJ9fQ.6tLnsjWPRCz0cW00j2nzV-SUk6GwrlYgxe9V_p5mhxU"
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
    "result": "Email and/or password not valid"
}
```
