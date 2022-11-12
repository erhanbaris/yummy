table! {
    user (id) {
        id -> Text,
        name ->  Text,
        email -> Text,
        device_id -> Text,
        custom_id -> Text,
        password -> Text,
        insert_date -> Integer,
        last_login_date -> Integer,
    }
}

table! {
    user_metadata (id) {
        id -> Text,
        user_id ->  Text,
        key -> Text,
        value -> Text,
    }
}

allow_tables_to_appear_in_same_query!(user, user_metadata,);
