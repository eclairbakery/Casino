use sqlx::FromRow;
use sqlx::Pool;
use sqlx::Sqlite;

#[derive(Debug, FromRow)]
pub struct Member {
    pub id: i64,
    pub cash: i64,
    pub bank: i64,
}

#[derive(Debug, FromRow)]
pub struct Timeouts {
    pub member_id: i64,
    pub last_crime: i64,
    pub last_rob: i64,
    pub last_slut: i64,
    pub last_work: i64,
}

pub struct DbManager {
    pool: Pool<Sqlite>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct FullMemberData {
    pub id: i64,
    pub cash: i64,
    pub bank: i64,
    pub last_crime: i64,
    pub last_rob: i64,
    pub last_slut: i64,
    pub last_work: i64,
}

impl DbManager {
    // --- base ---
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    // --- members ---
    pub async fn get_member(&self, user_id: i64) -> Result<Option<Member>, sqlx::Error> {
        sqlx::query_as::<_, Member>("SELECT * FROM members WHERE id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn ensure_member(&self, user_id: i64) -> Result<(Member, Timeouts), sqlx::Error> {
        let row = sqlx::query_as::<_, FullMemberData>(
            "SELECT m.id, m.cash, m.bank, t.last_crime, t.last_rob, t.last_slut, t.last_work 
             FROM members m 
             JOIN timeouts t ON m.id = t.member_id 
             WHERE m.id = ?"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(d) = row {
            return Ok((
                Member { id: d.id, cash: d.cash, bank: d.bank },
                Timeouts { 
                    member_id: d.id, 
                    last_crime: d.last_crime, 
                    last_rob: d.last_rob, 
                    last_slut: d.last_slut, 
                    last_work: d.last_work 
                }
            ));
        }

        let mut tx = self.pool.begin().await?;

        sqlx::query("INSERT OR IGNORE INTO members (id) VALUES (?)")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("INSERT OR IGNORE INTO timeouts (member_id) VALUES (?)")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok((
            Member { id: user_id, cash: 0, bank: 0 },
            Timeouts {
                member_id: user_id,
                last_crime: 0,
                last_rob: 0,
                last_slut: 0,
                last_work: 0,
            }
        ))
    }

    // --- timeouts ---
    pub async fn get_timeouts(&self, user_id: i64) -> Result<Option<Timeouts>, sqlx::Error> {
        sqlx::query_as::<_, Timeouts>("SELECT * FROM timeouts WHERE member_id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn update_timeout(&self, user_id: i64, activity: &str, timestamp: i64) -> Result<(), sqlx::Error> {
        let query_str = format!("UPDATE timeouts SET {} = ? WHERE member_id = ?", activity);
        
        sqlx::query(&query_str)
            .bind(timestamp)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }

    // --- cash ---
    pub async fn add_cash(&self, user_id: i64, amount: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE members SET cash = cash + ? WHERE id = ?")
            .bind(amount)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn remove_cash(&self, user_id: i64, amount: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE members SET cash = cash - ? WHERE id = ?")
            .bind(amount)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- bank money ---
    pub async fn add_bank_money(&self, user_id: i64, amount: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE members SET bank = bank + ? WHERE id = ?")
            .bind(amount)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn remove_bank_money(&self, user_id: i64, amount: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE members SET bank = bank - ? WHERE id = ?")
            .bind(amount)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}