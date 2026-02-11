use anyhow::Error;
use sqlx::Pool;
use sqlx::Sqlite;

use crate::services::database::models::*;

pub struct DbManager {
    pub pool: Pool<Sqlite>,
}

impl DbManager {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn ensure_member(&self, user_id: i64) -> Result<UserData, Error> {
        let user_row = sqlx::query_as::<_, User>("SELECT id, cash, bank FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        let timeouts_row = sqlx::query_as::<_, Timeouts>(
            "SELECT last_crime,last_rob, last_slut, last_work, last_hazarded FROM timeouts WHERE user_id = ?"
        )
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(user) = user_row {
            if let Some(timeouts) = timeouts_row {
                return Ok(UserData { user, timeouts });
            }
        }

        let mut tx = self.pool.begin().await?;

        sqlx::query("INSERT OR IGNORE INTO users (id) VALUES (?)")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("INSERT OR IGNORE INTO timeouts (user_id) VALUES (?)")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok((UserData {
            user: User {
                id: user_id,
                ..Default::default()
            },
            timeouts: Timeouts::default(),
        }))
    }

    pub async fn update_timeout(
        &self,
        user_id: i64,
        activity: &str,
        timestamp: i64,
    ) -> Result<(), Error> {
        let query_str = format!("UPDATE timeouts SET {} = ? WHERE user_id = ?", activity);

        sqlx::query(&query_str)
            .bind(timestamp)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn change_cash(&self, user_id: i64, amount: f64) -> Result<(), Error> {
        sqlx::query("UPDATE users SET cash = cash + ? WHERE id = ?")
            .bind(amount)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_top_members(&self, limit: i64) -> Result<Vec<User>, Error> {
        let users =
            sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY (cash + bank) DESC LIMIT ?")
                .bind(limit)
                .fetch_all(&self.pool)
                .await?;

        Ok(users)
    }

    pub async fn deposit(&self, user_id: i64, amount: f64) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await?;

        let row: (f64,) = sqlx::query_as("SELECT cash FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;

        if row.0 < amount {
            return Ok(false);
        }

        sqlx::query("UPDATE users SET cash = cash - ?, bank = bank + ? WHERE id = ?")
            .bind(amount)
            .bind(amount)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(true)
    }

    pub async fn withdraw(&self, user_id: i64, amount: f64) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let row: (f64,) = sqlx::query_as("SELECT bank FROM members WHERE id = ?")
            .bind(user_id)
            .fetch_one(&mut *tx)
            .await?;

        if row.0 < amount {
            return Err(Error::msg("Insufficient funds"));
        }

        sqlx::query("UPDATE users SET cash = cash + ?, bank = bank - ? WHERE id = ?")
            .bind(amount)
            .bind(amount)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn transfer(&self, victim_id: i64, thief_id: i64, amount: f64) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("UPDATE users SET cash = cash - ? WHERE id = ?")
            .bind(amount)
            .bind(victim_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE users SET cash = cash + ? WHERE id = ?")
            .bind(amount)
            .bind(thief_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn process_purchase(&self, user_id: i64, cost: f64) -> Result<bool, Error> {
        let mut transaction = self.pool.begin().await?;

        let result = sqlx::query("UPDATE users SET cash = cash - ? WHERE id = ? AND cash >= ?")
            .bind(cost)
            .bind(user_id)
            .bind(cost)
            .execute(&mut *transaction)
            .await?;

        if result.rows_affected() > 0 {
            transaction.commit().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
