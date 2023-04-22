use serde::{Deserialize, Serialize};

use crate::schema::{teams, users};

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct Team {
    pub id: i32,
    pub team_name: String,
    pub owner: String,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = teams)]
pub struct NewTeam {
    pub team_name: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, diesel::Queryable, Debug)]
pub struct User {
    pub email: String,
    pub display_name: String,
    pub team_id: Option<i32>,
}

#[derive(diesel::Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub display_name: String,
}
