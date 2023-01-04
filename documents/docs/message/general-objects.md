### :material-table: Meta

This area is used to store private or public information. Information can be kept dynamically and access to this information can be arranged. However, only certain data types are supported. number, boolean and string types are supported. nested declaration and array are not supported. It must be defined as a key-value. Value part may contain a value or if it is desired to determine the authorization level, it should be defined as an object and authorization information should be given. Access level of all created meta is defined as **0**.

When the query is made, meta information that has been assigned a lower authority than the user/room's authority can also be seen. In other words, if the user/room has the moderator authority, they can see all the metas with **Anonymous**, **Registered user**, **Friend**, **Me** and **Moderator** privileges.

If the **null** is assigned into the key, that key will be removed from room.

[:material-table: See the access level table.](#meta-access-level)

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

### :material-table: Meta access level

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