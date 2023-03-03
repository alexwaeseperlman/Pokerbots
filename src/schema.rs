// @generated automatically by Diesel CLI.

diesel::table! {
    teams (id) {
        id -> Int4,
        teamname -> Text,
    }
}

diesel::table! {
    users (email) {
        email -> Text,
        displayname -> Text,
        teamid -> Nullable<Int4>,
    }
}

diesel::joinable!(users -> teams (teamid));

diesel::allow_tables_to_appear_in_same_query!(
    teams,
    users,
);
