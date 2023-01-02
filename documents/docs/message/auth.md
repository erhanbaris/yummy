# Authentication related messages


---

## :fontawesome-solid-user-plus: Authenticate via email

=== ":inbox_tray: Request message"
    !!! success ""

        | Field name   | Type    | Required | Description                                    | Default value |
        |--------------|---------|----------|------------------------------------------------|---------------|
        | `type`       | string  | Y        | Value must be **Auth**                         |               |
        | `auth_type`  | string  | Y        | Value must be **Email**                        |               |
        | `email`      | string  | Y        | Authentication email address                   |               |
        | `password`   | string  | Y        | Authentication password                        |               |
        | `create`     | boolean | N        | If the user is not created yet, create new one | false         |

        **Example request:**
        ```json
        {
            "type": "Auth",
            "auth_type": "Email",
            "email": "erhanbaris@gmail.com",
            "password": "erhan",
            "create": true
        }
        ```
=== ":outbox_tray: Response message"
    Please check [Authenticate response message](#authenticate-response-message)

---

## :fontawesome-solid-user-plus: Authenticate via device id

=== ":inbox_tray: Request message"
    !!! success ""

        | Field name   | Type    | Required | Description                                    |
        |--------------|---------|----------|------------------------------------------------|
        | `type`       | string  | Y        | Value must be **Auth**                         |
        | `auth_type`  | string  | Y        | Value must be **DeviceId**                     |
        | `id`         | string  | Y        | Authentication device id                       |

        **Example request:**
        ```json
        {
            "type": "Auth",
            "auth_type": "DeviceId",
            "id": "1234567890"
        }
        ```
=== ":outbox_tray: Response message"

    Please check [Authenticate response message](#authenticate-response-message)

---

## :fontawesome-solid-user-plus: Authenticate via custom id

=== ":inbox_tray: Request message"

    !!! success ""

        | Field name   | Type    | Required | Description                                    |
        |--------------|---------|----------|------------------------------------------------|
        | `type`       | string  | Y        | Value must be **Auth**                         |
        | `auth_type`  | string  | Y        | Value must be **CustomId**                     |
        | `id`         | string  | Y        | Authentication device id                       |

        **Example request:**
        ```json
        {
            "type": "Auth",
            "auth_type": "CustomId",
            "id": "ABV123"
        }
        ```
=== ":outbox_tray: Response message"

    Please check [Authenticate response message](#authenticate-response-message)

---

## :material-shield-refresh-outline: Refreshing token

Regenerating token with new expire date.

=== ":inbox_tray: Request message"

    !!! success ""

        | Field name   | Type    | Required | Description                                    |
        |--------------|---------|----------|------------------------------------------------|
        | `type`       | string  | Y        | Value must be **Auth**                         |
        | `auth_type`  | string  | Y        | Value must be **Refresh**                      |
        | `token`      | string  | Y        | Valid Authentication token                     |

        **Example request:**
        ```json
        {
            "type": "Auth",
            "auth_type": "Refresh",
            "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2NzA1Mjg3MTEsInVzZXIiOnsiaWQiOiJiMDhkN2I3OS0xNDA1LTQxZGMtODJhMS02YTg4MjU3OWM3MmEiLCJzZXNzaW9uIjoiM2IxMzdjYWUtZmY1OC00NjY5LTg1YjctOWEyM2NiOGRiYzAxIiwibmFtZSI6bnVsbCwiZW1haWwiOiJlcmhhbmJhcmlzQGdtYWlsLmNvbSJ9fQ.6tLnsjWPRCz0cW00j2nzV-SUk6GwrlYgxe9V_p5mhxU"
        }
        ```

=== ":outbox_tray: Response message"
    Please check [Authenticate response message](#authenticate-response-message)

---

## :material-restore: Restoring session

The user should be restore token after reconnecting to the system. If timeout exceeded, the session will be terminated and active game and player will be informed.

=== ":inbox_tray: Request message"

    !!! success ""

        | Field name   | Type    | Required | Description                                    |
        |--------------|---------|----------|------------------------------------------------|
        | `type`       | string  | Y        | Value must be **Auth**                         |
        | `auth_type`  | string  | Y        | Value must be **Restore**                      |
        | `token`      | string  | Y        | Valid Authentication token                     |

        **Example request:**
        ```json
        {
            "type": "Auth",
            "auth_type": "Restore",
            "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2NzA1Mjg3MTEsInVzZXIiOnsiaWQiOiJiMDhkN2I3OS0xNDA1LTQxZGMtODJhMS02YTg4MjU3OWM3MmEiLCJzZXNzaW9uIjoiM2IxMzdjYWUtZmY1OC00NjY5LTg1YjctOWEyM2NiOGRiYzAxIiwibmFtZSI6bnVsbCwiZW1haWwiOiJlcmhhbmJhcmlzQGdtYWlsLmNvbSJ9fQ.6tLnsjWPRCz0cW00j2nzV-SUk6GwrlYgxe9V_p5mhxU"
        }
        ```
=== ":outbox_tray: Response message"

    Please check [Authenticate response message](#authenticate-response-message)

---

## :fontawesome-solid-user-slash: Logout

Terminates the current session and active game and player will be informed.

=== ":inbox_tray: Request message"
    !!! success ""

        | Field name   | Type    | Required | Description                                    |
        |--------------|---------|----------|------------------------------------------------|
        | `type`       | string  | Y        | Value must be **Auth**                         |
        | `auth_type`  | string  | Y        | Value must be **Logout**                       |

        **Example request:**
        ```json
        {
            "type": "Auth",
            "auth_type": "Logout"
        }
        ```

=== ":outbox_tray: Response message"

    !!! abstract ""
        === "Success"

            | Field name   | Type    | Nullable | Description                 |
            |--------------|---------|----------|-----------------------------|
            | `status`     | boolean | N        | Value should be **true**      |
            
            **Example response:**
            ```json
            {
                "status": true
            }
            ```

        === "Fail"

            | Field name   | Type    | Nullable | Description                 |
            |--------------|---------|----------|-----------------------------|
            | `status`     | boolean | N        | Value should be **false**     |
            | `error`      | string  | N        | Error message               |

            **Example response:**
            ```json
            {
                "status": false,
                "error" "User not logged in"
            }
            ```

---

## Authenticate response message

All authentication response message structure is the same.

!!! abstract ""
    === "Success"

        | Field name | Type    | Nullable | Description                       |
        |------------|---------|----------|-----------------------------------|
        | `status`   | boolean | N        | Value should be **true**          |
        | `type`     | string  | N        | Value should be **Authenticated** |
        | `token`    | string  | N        | User's authentication token       |

        **Example response**:

        ```json
        {
            "status": true,
            "type": "Authenticated",
            "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2NzI3NzE2NTQsInVzZXIiOnsiaWQiOiJiMDhkN2I3OS0xNDA1LTQxZGMtODJhMS02YTg4MjU3OWM3MmEiLCJzZXNzaW9uIjoiZWJiNWNkNzctM2M2Ni00NTQ2LTk2OGQtYTNjOGMwNTBiMjczIiwibmFtZSI6bnVsbCwiZW1haWwiOiJlcmhhbmJhcmlzQGdtYWlsLmNvbSIsInVzZXJfdHlwZSI6MX19.k2eM1xV4XnUx33f0pBVUD_lLgIcw0K1l2DOpJueG7g8"
        }
        ```

    === "Fail"

        | Field name   | Type    | Nullable | Description                 |
        |--------------|---------|----------|-----------------------------|
        | `status`     | boolean | N        | Value should be **false**   |
        | `error`      | string  | N        | Error message               |

        **Example response**:

        ```json
        {
            "status": false,
            "error" "Email and/or password not valid"
        }
        ```
