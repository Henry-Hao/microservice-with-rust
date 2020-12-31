table! {
    comments (id) {
        id -> Nullable<Integer>,
        uid -> Text,
        text -> Text,
    }
}

table! {
    users (id) {
        id -> Text,
        email -> Text,
        password -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    comments,
    users,
);
