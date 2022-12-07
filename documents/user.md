# User related messages

## Get my information

### Request message

| Field name | Type    | Required | Description           |
|------------|---------|----------|-----------------------|
| type       | string  | Y        | Value must be 'User'  |
| user_type  | string  | Y        | Value must be 'Me'    |

**Example request**

```json
{
    "type": "User",
    "auth_type": "Me"
}
```

[Success response](#user-information-success-response)

[Fail response](#fail-response)


## Get user's information

### Request message

| Field name | Type    | Required | Description           |
|------------|---------|----------|-----------------------|
| type       | string  | Y        | Value must be 'User'  |
| user_type  | string  | Y        | Value must be 'Me'    |
| user       | string  | Y        | User's unique id      |

**Example request**

```json
{
    "type": "User",
    "auth_type": "Get",
    "user": "258cd77c-1618-4c44-baff-6ec73c57fa85"
}
```

[Success response](#user-information-success-response)

[Fail response](#fail-response)

### User information success response


| Field name | Type            | Required | Description               |
|-----------:|-----------------|----------|---------------------------|
| status     | boolean         | Y        | Value should be 'true'    |
| result     | [UserInfoObject](#UserInfoObject)  | Y        | User's information object |

```json
{
    "status": true,
    "result": {
        "id": "cf20d9d2-e555-4008-886a-451c11dae64c",
        "name": "erhan",
        "email": "erhan@erhan.com",
        "device_id": "ABC123",
        "custom_id": "1234567890",
        "meta": {
            "user type": 8.0,
            "lat": 3.11133,
            "lon": 5.444,
            "me type": 9.0
        },
        "user_type": 1,
        "online": true,
        "insert_date": 1670446270,
        "last_login_date": 1670446270
    }
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


## Message objects

### UserInfoObject

It keeps the information about the user together. It is object type.
|      Field name | Type           | Nullable | Description             |
|----------------:|----------------|----------|-------------------------|
| id              | string         | N        | User's unique id        |
| name            | string         | Y        | User's name             |
| email           | string         | Y        | Email                   |
| device_id       | string         | Y        | Device id               |
| custom_id       | string         | Y        | Custom id               |
| meta            | [ResponseMetaObject](#ResponseMetaObject)     | Y        | Meta object             |
| user_type       | [UserType](#UserType) | N        | User's type information |
| online          | boolean        | N        |                         |
| insert_date     | number         | N        |                         |
| last_login_date | number         | N        |                         |


### ResponseMetaObject

It keeps dynamic informations about user. It is object type and value informations can only one of number, boolean or string.

```json
{
    "lat": 3.11133,
    "lon": 5.444,
    "admin": false,
    "city": "Copenhagen"
}
```


### UserType

| Name  | Value |
|-------|-------|
| User  | 1     |
| Mod   | 2     |
| Admin | 3     |
