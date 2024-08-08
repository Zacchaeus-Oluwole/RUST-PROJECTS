// pub mod model;

use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Deserialize, Serialize)]

pub struct Profile {
    pub id: i32,
    pub eid: String,
    pub ename: String,
    pub eemail: String,
    pub econtact: String,
}

#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct NewProfile {
    pub id: i32,
    pub eid: String,
    pub ename: String,
    pub eemail: String,
    pub econtact: String,
}