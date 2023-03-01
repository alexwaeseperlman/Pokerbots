use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(diesel::Queryable))]
pub struct Team {
    pub id: i32,
    pub teamname: String,
}
