use cfg_if::cfg_if;

use diesel::prelude::*;
#[derive(Queryable)]
pub(crate) struct Team {
    pub(crate) id: i32,
    pub(crate) teamname: String,
}
