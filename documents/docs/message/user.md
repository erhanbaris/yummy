# User related messages

---

## :fontawesome-solid-user: Get my information

=== ":inbox_tray: Request message"
    !!! success ""
        | Field name   | Type    | Required | Description             |
        |--------------|---------|----------|-------------------------|
        | `type`       | string  | Y        | Value must be **Me**    |

        **Example request:**

        ```json
        {
            "type": "Me"
        }
        ```

=== ":outbox_tray: Response message"
    Please check [User information response](#user-information-response)

---

## :fontawesome-solid-clipboard-user: Get user's information

=== ":inbox_tray: Request message"

    !!! success ""

        | Field name | Type    | Required | Description                  |
        |------------|---------|----------|------------------------------|
        | `type`       | string  | Y        | Value must be **GetUser**  |
        | `user_id`    | string  | Y        | User's unique id           |

        **Example request:**

        ```json
        {
            "type": "GetUser",
            "user_id": "258cd77c-1618-4c44-baff-6ec73c57fa85"
        }
        ```

=== ":outbox_tray: Response message"
    Please check [User information response](#user-information-response)

---

## :fontawesome-solid-user-pen: Information update

Update user information. Current implementation only allow to update own informations. Later on the system will have a support to update another user's informations. That is partially implemented but requires more changes and controls.

=== ":inbox_tray: Request message"
    !!! success ""
        | Field name    | Type                        | Nullable | Description                                                                                                                    |
        |---------------|-----------------------------|----------|--------------------------------------------------------------------------------------------------------------------------------|
        | `type`        | string                      | N        | Value must be **UpdateUser**                                                                                                   |
        | `name`        | string                      | Y        |                                                                                                                                |
        | `email`       | string                      | Y        |                                                                                                                                |
        | `password`    | string                      | Y        |                                                                                                                                |
        | `device_id`   | string                      | Y        |                                                                                                                                |
        | `custom_id`   | string                      | Y        |                                                                                                                                |
        | `user_type`   | [UserType](#usertype)       | Y        |                                                                                                                                |
        | `metas`       | [[Meta]](general-objects.md#meta)     | Y        | Array of [Meta](general-objects.md#meta) information. This is user based information and have access level to whom see that information. |
        | `meta_action` | [MetaAction](general-objects.md#meta-actions) | Y        | Default value is **0**                                                                                                         |
        
        **Example request:**

        === "Update all informations"
            ```json
            {
                "type": "UpdateUser",
                "name": "erhan",
                "email": "erhanbaris@gmail.com",
                "password": "12345",
                "device_id": "abc123",
                "custom_id": "1234567890",
                "user_type": 3,
                "meta": {
                    "lat": 123.0,
                    "lon": 321.0
                }
            }
            ```
        === "Only password change"
            ```json
            {
                "type": "UpdateUser",
                "password": "12345"
            }
            ```
        === "Only user type change"
            ```json
            {
                "type": "UpdateUser",
                "user_type": 3,
            }
            ```
        === "Only user type change"
            ```json
            {
                "type": "UpdateUser",
                "user_type": 3,
            }
            ```

=== ":outbox_tray: Response message"
    Please check [User information response](#user-information-response)

---

## User information response
!!! abstract ""
    === ":material-check: Success"
        | Field name        | Type                  | Nullable | Description                  |
        |-------------------|-----------------------|----------|------------------------------|
        | `status`          | boolean               | N        | Value should be **true**     |
        | `type`            | string                | N        | Value should be **UserInfo** |
        | `id`              | string                | N        | User's unique id             |
        | `name`            | string                | Y        | User's name                  |
        | `email`           | string                | Y        | Email                        |
        | `device_id`       | string                | Y        | Device id                    |
        | `custom_id`       | string                | Y        | Custom id                    |
        | `meta`            | [UserMeta](#usermeta) | Y        | Meta object                  |
        | `user_type`       | [UserType](#usertype) | N        | User's type information      |
        | `online`          | boolean               | N        |                              |
        | `insert_date`     | number                | N        |                              |
        | `last_login_date` | number                | N        |                              |

        **Example response**:
        ```json
        {
            "status": true,
            "type": "UserInfo",
            "id": "b08d7b79-1405-41dc-82a1-6a882579c72a",
            "name": null,
            "email": "erhanbaris@gmail.com",
            "device_id": null,
            "custom_id": null,
            "meta": null,
            "user_type": 1,
            "online": true,
            "insert_date": 1670062718,
            "last_login_date": 1672694548
        }
        ```

    === ":octicons-x-16: Fail"

        | Field name   | Type    | Nullable | Description                   |
        |--------------|---------|----------|-------------------------------|
        | `status`     | boolean | N        | Value should be **false**     |
        | `error`      | string  | N        | Error message                 |

        **Example response**:
        ```json
        {
            "status": false,
            "error": "Email and/or password not valid"
        }
        ```
