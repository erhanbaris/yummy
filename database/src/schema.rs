table! {
    user (id) {
        id -> Text,
        name ->  Text,
        email -> Text,
        password -> Text,
        insert_date -> Integer,
        last_login_date -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(user,);
