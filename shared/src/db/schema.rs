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
    game_states (game, step) {
        game -> Text,
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
        round -> Text,
        action_time -> Int4,
        last_action -> Text,
    }
}

diesel::table! {
    games (id) {
        id -> Text,
        defender -> Int4,
        challenger -> Int4,
        defender_score -> Nullable<Int4>,
        created -> Int8,
        error_type -> Nullable<Text>,
        challenger_score -> Nullable<Int4>,
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

diesel::joinable!(game_states -> games (game));
diesel::joinable!(team_invites -> teams (team));
diesel::joinable!(teams -> users (owner));

diesel::allow_tables_to_appear_in_same_query!(bots, game_states, games, team_invites, teams, users,);
