use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub struct Model {
    db_pool: SqlitePool,
}

impl Model {
    pub async fn new() -> Self {
        // TODO: Handle errors properly
        
        // Connect to SQLite (creates data.db if it doesn't exist)
        let db_pool = SqlitePool::connect_with(
            SqliteConnectOptions::new()
            .filename("data.db")
            .create_if_missing(true)
        ).await.unwrap();

        // Create white monster counter table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS white_monster_counter (
                user_id TEXT NOT NULL UNIQUE,
                count INT NOT NULL
            )
            "#,
        )
        .execute(&db_pool)
        .await.unwrap();

        // Create bank table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bank (
                user_id TEXT NOT NULL UNIQUE,
                balance REAL NOT NULL
            )
            "#,
        )
        .execute(&db_pool)
        .await.unwrap();
        
        Self { db_pool }
    }

    pub async fn wm_counters(&self) -> (u32, Vec<(u64, u32)>) {
        // FIXME: Handle results properly
        let wm_data: Vec<(String, u32)> = sqlx::query_as(
            r#"
            SELECT user_id, count
            FROM white_monster_counter
            ORDER BY count DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap();

        let total_count = wm_data.iter().map(|(_, count)| *count).sum();
        let wm_data_converted = wm_data
            .iter()
            .map(|(user_id, count)| (user_id.parse::<u64>().unwrap(), *count))
            .collect::<Vec<(u64, u32)>>();
        (total_count, wm_data_converted)
    }

    pub async fn inc_wm_counter(&self, user_id: u64, amount: u32) -> u32 {
        // FIXME: Handle results properly
        
        let wm_counter: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO white_monster_counter(user_id, count)
            VALUES($1, $2) ON CONFLICT(user_id) DO
            UPDATE SET count = count + $2
            RETURNING count;
            "#
        )
        .bind(user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await
        .unwrap();
        
        u32::try_from(wm_counter).unwrap()
    }

    pub async fn dec_wm_counter(&self, user_id: u64, amount: u32) -> u32 {
        // FIXME: Handle results properly
        let wm_counter: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO white_monster_counter(user_id, count)
            VALUES($1, $2) ON CONFLICT(user_id) DO
            UPDATE SET count = count - $2
            RETURNING count;
            "#
        )
        .bind(user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await
        .unwrap();
        
        u32::try_from(wm_counter).unwrap()
    }

    pub async fn balance(&self, user_id: u64) -> Option<f64> {
        let res = sqlx::query_scalar(
            r#"
            SELECT balance FROM bank WHERE user_id = $1
            "#
        )
        .bind(user_id.to_string())
        .fetch_one(&self.db_pool)
        .await;

        match res {
            Ok(balance) => Some(balance),
            Err(sqlx::Error::RowNotFound) => None,
            Err(err) => panic!("{err}"),
        }
    }

    pub async fn create_bank_account(&self, user_id: u64) -> bool {
        // By default user starts with 0 eur 
        let res = sqlx::query(
            r#"
            INSERT INTO bank VALUES ($1, 0)
            "#
        )
        .bind(user_id.to_string())
        .execute(&self.db_pool)
        .await;

        res.is_ok()
    }

    pub async fn bank_account_exists(&self, user_id: u64) -> bool {
        let exists: (i32,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM bank WHERE user_id = $1)")
            .bind(user_id.to_string())
            .fetch_one(&self.db_pool)
            .await
            .unwrap();

        exists.0 == 1
    }

    // Returns None if src_user has insuficient funds
    pub async fn give(&self, src_user_id: u64, dst_user_id: u64, amount: f64) -> Option<(f64, f64)> {
        // DB PRECONDITION: - src_user_id and dst_user_id exist in the bank table
        assert!(amount > 0f64);

        // Update src_user balance
        let src_new_balance_res = sqlx::query_scalar(
            r#"
            UPDATE bank SET balance = balance - $2
            WHERE user_id = $1 AND balance >= $2
            RETURNING balance
            "#
        )
        .bind(src_user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await;

        
        let src_new_balance = match src_new_balance_res {
            Ok(Some(src_new_balance)) => src_new_balance,
            Err(sqlx::Error::RowNotFound) => {return None;},
            _ => panic!("Unexpected behaviour"),
        };
        
        // Update dst_user balance
        let dst_new_balance_opt: Option<f64> = sqlx::query_scalar(
            r#"
            UPDATE bank SET balance = balance + $2
            WHERE user_id = $1
            RETURNING balance
            "#
        )
        .bind(dst_user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await
        .unwrap();

        Some((src_new_balance, dst_new_balance_opt.unwrap()))
    }

    pub async fn bless(&self, dst_user_id: u64, amount: f64) -> Option<f64> {
        let new_balance_res = sqlx::query_scalar(
            r#"
            UPDATE bank SET balance = MAX(balance + $2, 0)
            WHERE user_id = $1
            RETURNING balance
            "#
        )
        .bind(dst_user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await;

        match new_balance_res {
            Ok(new_balance) => Some(new_balance),
            Err(sqlx::Error::RowNotFound) => None,
            Err(err) => panic!("{err}"),
        }
    }

    pub async fn leaderboard(&self) -> Vec<(u64, f64)> {
        let bank_data: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT user_id, balance
            FROM bank
            ORDER BY balance DESC
            "#
        )
        .fetch_all(&self.db_pool)
        .await
        .unwrap();

        let bank_data_converted = bank_data
            .iter()
            .map(|(user_id, count)| (user_id.parse::<u64>().unwrap(), *count))
            .collect::<Vec<(u64, f64)>>();
        bank_data_converted
    }
}