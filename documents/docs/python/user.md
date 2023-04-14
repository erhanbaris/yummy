# User API's

## Get user information

=== "Code"
    !!! success ""

        ```python
        def pre_get_user_information(model):
            pass

        def post_get_user_information(model, successed):
            pass
        ```
=== "Model details"
    !!! success ""
        | Name                   | Return                | Description                |
        |------------------------|-----------------------|----------------------------|
        | `get_query()`          | `GetUserInformation`  | Get user information query |

        ### GetUserInformation
        | Name                   | Return                | Description                                                        |
        |------------------------|-----------------------|--------------------------------------------------------------------|
        | `get_type()`           | `string`              | Message type. Message types are: 'Me', 'User' and 'UserViaSystem'. |
        | `as_table()`           | `Table`              | Message type. Message types are: 'Me', 'User' and 'UserViaSystem'.  |

        `as_table()` return `Table` and `Table` contains following keys: **type**, **user_id**, **session_id**, **requester_user_id**, **requester_session_id** based on `get_type()` information.
