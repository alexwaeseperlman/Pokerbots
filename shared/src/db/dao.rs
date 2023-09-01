use crate::db::{models::*, schema, schema_aliases};
use diesel::PgConnection;
use diesel::{dsl::*, prelude::*};

pub mod bots;
pub mod games;
