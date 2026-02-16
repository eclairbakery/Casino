use sqlx::FromRow;

#[derive(Default, FromRow)]
pub struct User {
    pub id: i64,
    pub cash: i64,
    pub bank: i64,
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
