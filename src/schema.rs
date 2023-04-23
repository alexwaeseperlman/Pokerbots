// @generated automatically by Diesel CLI.

diesel::table! {
    team_invites (id) {
        id -> Int4,
        teamid -> Int4,
        invite_code -> Int8,
        expires -> Int8,
        used -> Bool,
    }
}

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

diesel::joinable!(team_invites -> teams (teamid));

diesel::allow_tables_to_appear_in_same_query!(
    team_invites,
    teams,
    users,
);
