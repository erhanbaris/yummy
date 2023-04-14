# Auth API's

## Auth via email

=== "Code"
    !!! success ""

        ```python
        def pre_email_auth(model):
            pass

        def post_email_auth(model, successed):
            pass
        ```
=== "Model details"
    !!! success ""
        | Name                   | Return    | Description                                                                   |
        |------------------------|-----------|-------------------------------------------------------------------------------|
        | `get_user_id()`        | `string`  | Get authenticated user id. If user not authenticated, value will be empty.    |
        | `get_session_id()`     | `string`  | Get authenticated session id. If user not authenticated, value will be empty. |
        | `get_email()`          | `string`  | User's emaill address.                                                        |
        | `get_password()`       | `string`  | User's password                                                               |
        | `get_create()`         | `boolean` | If the user not available on the system, create the user.                     |
        | `set_email(string)`    |           | Set user's email address.                                                     |
        | `set_password(string)` |           | Set user's password address.                                                  |
        | `set_create(boolean)`  |           | Set create information. Please check `get_create` for more information.       |

## Auth via device id

=== "Code"
    !!! success ""

        ```python
        def pre_deviceid_auth(model):
            pass

        def post_deviceid_auth(model, successed):
            pass
        ```
=== "Model details"
    !!! success ""
        | Name                   | Return    | Description                                                                   |
        |------------------------|-----------|-------------------------------------------------------------------------------|
        | `get_user_id()`        | `string`  | Get authenticated user id. If user not authenticated, value will be empty.    |
        | `get_session_id()`     | `string`  | Get authenticated session id. If user not authenticated, value will be empty. |
        | `get_id()`             | `string`  | Get device id.                                                                |
        | `set_id(string)`       |           | Set device id.                                                                |


## Auth via custom id

=== "Code"
    !!! success ""

        ```python
        def pre_customid_auth(model):
            pass

        def post_customid_auth(model, successed):
            pass
        ```
=== "Model details"
    !!! success ""
        | Name                   | Return    | Description                                                                   |
        |------------------------|-----------|-------------------------------------------------------------------------------|
        | `get_user_id()`        | `string`  | Get authenticated user id. If user not authenticated, value will be empty.    |
        | `get_session_id()`     | `string`  | Get authenticated session id. If user not authenticated, value will be empty. |
        | `get_id()`             | `string`  | Get custom id.                                                                |
        | `set_id(string)`       |           | Set custom id.                                                                |


## Logout

=== "Code"
    !!! success ""

        ```python
        def pre_logout(model):
            pass

        def post_logout(model, successed):
            pass
        ```
=== "Model details"
    !!! success ""
        | Name                   | Return    | Description                                                                   |
        |------------------------|-----------|-------------------------------------------------------------------------------|
        | `get_user_id()`        | `string`  | Get authenticated user id. If user not authenticated, value will be empty.    |
        | `get_session_id()`     | `string`  | Get authenticated session id. If user not authenticated, value will be empty. |


## Restore token

=== "Code"
    !!! success ""

        ```python
        def pre_restore_token(model):
            pass

        def post_restore_token(model, successed):
            pass
        ```
=== "Model details"
    !!! success ""
        | Name                   | Return    | Description                                                                   |
        |------------------------|-----------|-------------------------------------------------------------------------------|
        | `get_user_id()`        | `string`  | Get authenticated user id. If user not authenticated, value will be empty.    |
        | `get_session_id()`     | `string`  | Get authenticated session id. If user not authenticated, value will be empty. |
        | `get_token()`          | `string`  | Get token.                                                                    |
        | `set_token()`          |           | Set token.                                                                    |


## Refresh token

=== "Code"
    !!! success ""

        ```python
        def pre_refresh_token(model):
            pass

        def post_refresh_token(model, successed):
            pass
        ```
=== "Model details"
    !!! success ""
        | Name                   | Return    | Description                                                                   |
        |------------------------|-----------|-------------------------------------------------------------------------------|
        | `get_user_id()`        | `string`  | Get authenticated user id. If user not authenticated, value will be empty.    |
        | `get_session_id()`     | `string`  | Get authenticated session id. If user not authenticated, value will be empty. |
        | `get_token()`          | `string`  | Get token.                                                                    |
        | `set_token()`          |           | Set token.                                                                    |

