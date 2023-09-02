// @generated automatically by Diesel CLI.

diesel::table! {
    auth (email) {
        email -> Text,
        mangled_password -> Nullable<Text>,
        email_verification_link -> Nullable<Text>,
        email_verification_link_expiration -> Nullable<Timestamp>,
        password_reset_link -> Nullable<Text>,
        password_reset_link_expiration -> Nullable<Timestamp>,
        email_confirmed -> Bool,
        is_admin -> Bool,
    }
}

diesel::table! {
    bots (id) {
        id -> Int4,
        team -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        created -> Int8,
        uploaded_by -> Text,
        build_status -> Int4,
        deleted_at -> Nullable<Int8>,
        rating -> Float4,
    }
}

diesel::table! {
    game_results (id) {
        id -> Text,
        challenger_rating_change -> Float4,
        defender_rating_change -> Float4,
        defender_score -> Int4,
        challenger_score -> Int4,
        error_type -> Nullable<Text>,
        error_bot -> Nullable<Int4>,
        updated_at -> Int8,
        defender_rating -> Float4,
        challenger_rating -> Float4,
    }
}

diesel::table! {
    games (id) {
        id -> Text,
        defender -> Int4,
        challenger -> Int4,
        created -> Int8,
        defender_rating -> Float4,
        challenger_rating -> Float4,
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
    }
}

diesel::joinable!(game_results -> games (id));
diesel::joinable!(team_invites -> teams (team));
diesel::joinable!(teams -> users (owner));

diesel::allow_tables_to_appear_in_same_query!(
    auth,
    bots,
    game_results,
    games,
    team_invites,
    teams,
    users,
);
