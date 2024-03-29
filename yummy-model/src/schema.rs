
table! {
    user {
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
    user_meta {
        id -> Text,
        user_id ->  Text,
        key -> Text,
        value -> Text,
        meta_type -> Integer,
        access -> Integer,
        insert_date -> Integer,
    }
}

table! {
    room {
        id -> Text,
        name ->  Nullable<Text>,
        description ->  Nullable<Text>,
        max_user -> Integer,
        password -> Nullable<Text>,
        access_type -> Integer,
        join_request -> Integer,
        insert_date -> Integer,
    }
}

table! {
    room_tag {
        id -> Text,
        room_id -> Text,
        tag -> Text,
        insert_date -> Integer,
    }
}

table! {
    room_user {
        id -> Text,
        room_id -> Text,
        user_id -> Text,
        room_user_type -> Integer,
        insert_date -> Integer,
    }
}

table! {
    room_user_request {
        id -> Text,
        room_id -> Text,
        user_id -> Text,
        room_user_type -> Integer,
        status -> Bool,
        status_updater_user_id -> Nullable<Text>,
        insert_date -> Integer,
    }
}

table! {
    room_user_ban {
        id -> Text,
        room_id -> Text,
        user_id -> Text,
        blocker_user_id -> Text,
        insert_date -> Integer,
    }
}

table! {
    room_meta {
        id -> Text,
        room_id ->  Text,
        key -> Text,
        value -> Text,
        meta_type -> Integer,
        access -> Integer,
        insert_date -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(user, user_meta, room, room_tag, room_user, room_meta, room_user_request, room_user_ban);
