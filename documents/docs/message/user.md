# User related messages

---

## :fontawesome-solid-user: Get my information

### Request message

!!! success ""
    | Field name   | Type    | Required | Description             |
    |--------------|---------|----------|-------------------------|
    | `type`       | string  | Y        | Value must be **User**  |
    | `user_type`  | string  | Y        | Value must be **Me**    |

    **Example request:**

    ```json
    {
        "type": "User",
        "auth_type": "Me"
    }
    ```

### Response message

Please check [User information response](#user-information-response)

---

## :fontawesome-solid-clipboard-user: Get user's information

### Request message

!!! success ""

    | Field name | Type    | Required | Description               |
    |------------|---------|----------|---------------------------|
    | `type`       | string  | Y        | Value must be **User**  |
    | `user_type`  | string  | Y        | Value must be **Me**    |
    | `user`       | string  | Y        | User's unique id        |

    **Example request:**

    ```json
    {
        "type": "User",
        "auth_type": "Get",
        "user": "258cd77c-1618-4c44-baff-6ec73c57fa85"
    }
    ```

### Response message

Please check [User information response](#user-information-response)

---

## :fontawesome-solid-user-pen: Information update

Update user information. Current implementation only allow to update own informations. Later on the system will have a support to update another user's informations. That is partially implemented but requires more changes and controls.

### Request message

!!! success ""
    | Field name    | Type                        | Nullable | Description                                                                                                                    |
    |---------------|-----------------------------|----------|--------------------------------------------------------------------------------------------------------------------------------|
    | `type`        | string                      | N        | Value must be **User**                                                                                                         |
    | `auth_type`   | string                      | N        | Value must be **Update**                                                                                                       |
    | `name`        | string                      | Y        |                                                                                                                                |
    | `email`       | string                      | Y        |                                                                                                                                |
    | `password`    | string                      | Y        |                                                                                                                                |
    | `device_id`   | string                      | Y        |                                                                                                                                |
    | `custom_id`   | string                      | Y        |                                                                                                                                |
    | `type`        | [UserType](#usertype)       | Y        |                                                                                                                                |
    | `meta`        | [[UserMeta]](#usermeta)     | Y        | Array of [UserMeta](#usermeta) information. This is user based information and have access level to whom see that information. |
    | `meta_action` | [MetaAction](#meta-actions) | Y        | Default value is **0**                                                                                                         |
    
    **Example request:**

    === "Update all informations"
        ```json
        {
            "type": "User",
            "user_type": "Update",
            "name": "erhan",
            "email": "erhanbaris@gmail.com",
            "password": "12345",
            "device_id": "abc123",
            "custom_id": "1234567890",
            "type": 3,
            "meta": {
                "lat": 123.0,
                "lon": 321.0
            }
        }
        ```
    === "Only password change"
        ```json
        {
            "type": "User",
            "user_type": "Update",
            "password": "12345"
        }
        ```
    === "Only user type change"
        ```json
        {
            "type": "User",
            "user_type": "Update",
            "type": 3,
        }
        ```
    === "Only user type change"
        ```json
        {
            "type": "User",
            "user_type": "Update",
            "type": 3,
        }
        ```

### Response message

Please check [User information response](#user-information-response)

---

## User information response

!!! abstract ""
    === "Success"
        | Field name   | Type                               | Required | Description                 |
        |--------------|------------------------------------|----------|-----------------------------|
        | `status`     | boolean                            | Y        | Value should be **true**    |
        | `result`     | [UserInfoObject](#userinfoobject)  | Y        | User's information object   |

        **Example response**:
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

    === "Fail"

        | Field name   | Type    | Required | Description                   |
        |--------------|---------|----------|-------------------------------|
        | `status`     | boolean | Y        | Value should be **false**     |
        | `result`     | string  | Y        | Error message                 |

        **Example response**:
        ```json
        {
            "status": false,
            "result": "Email and/or password not valid"
        }
        ```


# Message objects

### :material-table:  UserInfoObject

It keeps the information about the user together. It is object type.

|      Field name   | Type                                          | Nullable | Description             |
|-------------------|-----------------------------------------------|----------|-------------------------|
| `id`              | string                                        | N        | User's unique id        |
| `name`            | string                                        | Y        | User's name             |
| `email`           | string                                        | Y        | Email                   |
| `device_id`       | string                                        | Y        | Device id               |
| `custom_id`       | string                                        | Y        | Custom id               |
| `meta`            | [UserMeta](#usermeta)                         | Y        | Meta object             |
| `user_type`       | [UserType](#usertype)                         | N        | User's type information |
| `online`          | boolean                                       | N        |                         |
| `insert_date`     | number                                        | N        |                         |
| `last_login_date` | number                                        | N        |                         |

### :material-table: UserType
It is the type of authorization at the entire system level of the user. The administrator and moderator levels differ among themselves, and the administrator level is the highest level that can be recognized.

| Value   | Description |
|:-------:|-------------|
| `1`     | User        |
| `2`     | Mod         |
| `3`     | Admin       |

### :material-table: UserMeta

This area is used to store user-based private or public information. Information can be kept dynamically and access to this information can be arranged. However, only certain data types are supported. number, boolean and string types are supported. nested declaration and array are not supported. It must be defined as a key-value. Value part may contain a value or if it is desired to determine the authorization level, it should be defined as an object and authorization information should be given. Access level of all created meta is defined as **0**.

When the query is made, meta information that has been assigned a lower authority than the user's authority can also be seen. In other words, if the user has the moderator authority, they can see all the metas with **Anonymous**, **Registered user**, **Friend**, **Me** and **Moderator** privileges.

If the **null** is assigned into the key, that key will be removed from user.

[:material-table: See the access level table.](#user-meta-access-level)

!!! success "Examples"
    === "Single definition"
        ```json
        {
            "location": "Copenhagen"
        }
        ```
    === "Multiple definition"
        ```json
        {
            "location": "Copenhagen",
            "age": 18,
            "maried": true
        }
        ```
    === "Definition with access level"
        ```json
        {
            "location": {
                "access": 4,
                "value": "Copenhagen"
            },
            "age": 18,
            "maried": true
        }
        ```
    === "Remove meta from user"
        ```json
        {
            "location": null
        }
        ```

### :material-table: User meta access level

| Value   | Information     |
|:-------:|-----------------|
| `0`     | Anonymous       |
| `1`     | Registered user |
| `2`     | Friend          |
| `3`     | Me              |
| `4`     | Moderator       |
| `5`     | Admin           |
| `6`     | System          |


### :material-table: Meta actions
It is the choice of algorithm to be used to add or delete new meta.

| Value   | Information                                                    |
|:-------:|----------------------------------------------------------------|
| `0`     | Only add new item or update                                    |
| `1`     | Add new item or update then remove unused metas                |
| `2`     | Remove all metas. Note: new meta definitions will be discarded |