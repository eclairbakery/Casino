use sqlx::{FromRow, Pool, Sqlite};

pub struct DbManager {
    pub pool: Pool<Sqlite>,
}

#[derive(Default, FromRow)]
pub struct User {
    pub id: i64,
    pub cash: f64,
    pub bank: f64,
}

#[derive(Default, FromRow)]
pub struct Timeouts {
    pub last_crime: i64,
    pub last_rob: i64,
    pub last_slut: i64,
    pub last_work: i64,
    pub last_hazarded: i64,
}

#[derive(FromRow)]
pub struct UserData {
    pub user: User,
    pub timeouts: Timeouts,
}
