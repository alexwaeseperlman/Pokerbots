use crate::schema::{teams, users};
use cfg_if::cfg_if;
use diesel::associations::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct Team {
    pub id: i32,
    pub teamname: String,
    pub owner: String,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = teams)]
pub struct NewTeam {
    pub teamname: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct User {
    pub email: String,
    pub displayname: String,
    pub team: Option<i32>,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub displayname: String,
}
