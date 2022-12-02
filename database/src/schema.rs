
table! {
    user (id) {
        id -> Text,
        name ->  Nullable<Text>,
        email -> Nullable<Text>,
        device_id -> Nullable<Text>,
        custom_id -> Nullable<Text>,
        password -> Nullable<Text>,
        user_type -> Integer,
        insert_date -> Integer,
        last_login_date -> Integer,
    }
}

table! {
    user_meta (id) {
        id -> Text,
        user_id ->  Text,
        key -> Text,
        value -> Text,
        meta_type -> Integer,
        access -> Integer,
        insert_date -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(user, user_meta,);
