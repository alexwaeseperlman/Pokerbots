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
        build_status -> Int4,
        deleted_at -> Nullable<Int8>,
    }
}

diesel::table! {
    games (id) {
        id -> Text,
        defender -> Int4,
        challenger -> Int4,
        defender_score -> Nullable<Int4>,
        challenger_score -> Nullable<Int4>,
        created -> Int8,
        error_type -> Nullable<Text>,
        error_bot -> Nullable<Int4>,
    }
}

diesel::table! {
    team_invites (code) {
        code -> Text,
        team -> Int4,
        expires -> Int8,
    }
}

diesel::table! {
    teams (id) {
        id -> Int4,
        name -> Text,
        owner -> Text,
        score -> Nullable<Int4>,
        active_bot -> Nullable<Int4>,
        deleted_at -> Nullable<Int8>,
    }
}

diesel::table! {
    users (email) {
        email -> Text,
        display_name -> Text,
        team -> Nullable<Int4>,
        is_admin -> Bool,
    }
}

diesel::joinable!(team_invites -> teams (team));
diesel::joinable!(teams -> users (owner));

diesel::allow_tables_to_appear_in_same_query!(bots, games, team_invites, teams, users,);
