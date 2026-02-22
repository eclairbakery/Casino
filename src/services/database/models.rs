use sqlx::{FromRow, Pool, Sqlite};

#[derive(Default, FromRow)]
pub struct User {
    pub id: i64,
    pub cash: i64,
    pub bank: i64,
}

impl User {
    pub async fn change_cash(&self, pool: &Pool<Sqlite>, amount: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET cash = cash + ? WHERE id = ?")
            .bind(amount)
            .bind(self.id)
            .execute(pool)
            .await?;

        Ok(())
    }
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
