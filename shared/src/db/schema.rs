// @generated automatically by Diesel CLI.

diesel::table! {
    auth (email) {
        email -> Text,
        mangled_password -> Text,
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
        score -> Float4,
        created -> Int8,
        uploaded_by -> Text,
        build_status -> Int4,
    }
}

diesel::table! {
    games (id) {
        id -> Text,
        defender -> Int4,
        challenger -> Int4,
        score_change -> Nullable<Int4>,
        created -> Int8,
        error_type -> Nullable<Text>,
        error_message -> Nullable<Text>,
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
    }
}

diesel::joinable!(team_invites -> teams (teamid));
diesel::joinable!(teams -> bots (active_bot));

diesel::allow_tables_to_appear_in_same_query!(
    auth,
    bots,
    games,
    team_invites,
    teams,
    users,
);
diesel::alias!(
    bots as defender_bots: DefenderBotsAlias,
    bots as challenger_bots: ChallengerBotsAlias,
    teams as defender_teams: DefenderTeamsAlias,
    teams as challenger_teams: ChallengerTeamsAlias
);
