// @generated automatically by Diesel CLI.

diesel::table! {
    bots (id) {
        id -> Int4,
        team -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        score -> Float4,
        created -> Int8,
        uploaded_by -> Text,
    }
}

diesel::table! {
    games (id) {
        id -> Text,
        bot_a -> Int4,
        bot_b -> Int4,
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
        active_bot -> Nullable<Int4>,
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

diesel::joinable!(teams -> bots (active_bot));

diesel::allow_tables_to_appear_in_same_query!(
    bots,
    games,
    team_invites,
    teams,
    users,
);
