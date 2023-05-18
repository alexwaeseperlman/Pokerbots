// @generated automatically by Diesel CLI.

diesel::table! {
    games (id) {
        id -> Text,
        teama -> Int4,
        teamb -> Int4,
        score_change -> Nullable<Int4>,
        created -> Int8,
    }
}

diesel::table! {
    team_invites (invite_code) {
        invite_code -> Text,
        teamid -> Int4,
        expires -> Int8,
    }
}

diesel::table! {
    teams (id) {
        id -> Int4,
        team_name -> Text,
        owner -> Text,
        score -> Nullable<Int4>,
    }
}

diesel::table! {
    users (email) {
        email -> Text,
        display_name -> Text,
        team_id -> Nullable<Int4>,
        is_admin -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    games,
    team_invites,
    teams,
    users,
);
