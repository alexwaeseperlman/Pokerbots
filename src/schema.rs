// @generated automatically by Diesel CLI.

diesel::table! {
    teams (id) {
        id -> Int4,
        team_name -> Text,
        owner -> Text,
    }
}

diesel::table! {
    users (email) {
        email -> Text,
        display_name -> Text,
        team_id -> Nullable<Int4>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    teams,
    users,
);
