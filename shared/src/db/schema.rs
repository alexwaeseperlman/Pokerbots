// @generated automatically by Diesel CLI.

diesel::table! {
    auth (id) {
        email -> Text,
        mangled_password -> Nullable<Text>,
        email_verification_link -> Nullable<Text>,
        email_verification_link_expiration -> Nullable<Timestamp>,
        password_reset_link -> Nullable<Text>,
        password_reset_link_expiration -> Nullable<Timestamp>,
        email_confirmed -> Bool,
        is_admin -> Bool,
        id -> Uuid,
    }
}

diesel::table! {
    bots (id) {
        id -> Int4,
        team -> Int4,
        name -> Text,
        description -> Nullable<Text>,
        created -> Int8,
        uploaded_by -> Uuid,
        build_status -> Int4,
        deleted_at -> Nullable<Int8>,
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
        updated_at -> Int8,
        defender_rating -> Float4,
        challenger_rating -> Float4,
    }
}

diesel::table! {
    game_states (game_id, step) {
        game_id -> Text,
        step -> Int4,
        challenger_stack -> Int4,
        defender_stack -> Int4,
        challenger_pushed -> Int4,
        defender_pushed -> Int4,
        challenger_hand -> Text,
        defender_hand -> Text,
        flop -> Nullable<Text>,
        turn -> Nullable<Text>,
        river -> Nullable<Text>,
        button -> Text,
        sb -> Text,
        action_time -> Int4,
        last_action -> Text,
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
        rated -> Bool,
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
        owner -> Uuid,
        active_bot -> Nullable<Int4>,
        deleted_at -> Nullable<Int8>,
        rating -> Float4,
    }
}

diesel::table! {
    user_profiles (id) {
        #[max_length = 255]
        first_name -> Varchar,
        #[max_length = 255]
        last_name -> Varchar,
        #[max_length = 255]
        country -> Nullable<Varchar>,
        #[max_length = 255]
        school -> Varchar,
        #[max_length = 255]
        linkedin -> Nullable<Varchar>,
        #[max_length = 255]
        github -> Nullable<Varchar>,
        id -> Uuid,
    }
}

diesel::table! {
    users (id) {
        display_name -> Text,
        team -> Nullable<Int4>,
        id -> Uuid,
    }
}

diesel::joinable!(bots -> auth (uploaded_by));
diesel::joinable!(game_results -> games (id));
diesel::joinable!(game_states -> games (game_id));
diesel::joinable!(team_invites -> teams (team));
diesel::joinable!(teams -> bots (active_bot));
diesel::joinable!(user_profiles -> auth (id));
diesel::joinable!(users -> auth (id));

diesel::allow_tables_to_appear_in_same_query!(
    auth,
    bots,
    game_results,
    game_states,
    games,
    team_invites,
    teams,
    user_profiles,
    users,
);
