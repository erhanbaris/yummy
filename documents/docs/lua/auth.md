# Auth API's

## Auth via email

=== "Code"
    !!! success ""

        ```lua
        function pre_email_auth(model)
        end

        function post_email_auth(model, successed)
        end
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