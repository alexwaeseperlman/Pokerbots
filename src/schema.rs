use cfg_if::cfg_if;
cfg_if! {
if #[cfg(feature="ssr")] {
// @generated automatically by Diesel CLI.

diesel::table! {
    teams (id) {
        id -> Integer,
        teamname -> Text,
    }
}

diesel::table! {
    users (email) {
        email -> Text,
        teamID -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(teams, users,);

}
}
