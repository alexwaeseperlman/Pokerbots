use crate::db::models::{BotWithTeam, Team};
use crate::db::schema;
use diesel::prelude::*;
use diesel::PgConnection;

pub mod bots;
