use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub struct Model {
    db_pool: SqlitePool,
}

impl Model {
    pub async fn new() -> Self {
        // Connect to SQLite (creates data.db if it doesn't exist)
        let db_pool = SqlitePool::connect_with(
            SqliteConnectOptions::new()
            .filename("data.db")
            .create_if_missing(true)
        ).await.unwrap();

        Self { db_pool }
    }

    pub async fn wm_counters(&self) -> (u32, Vec<(u64, u32)>) {
        // FIXME: Handle results properly
        let wm_data: Vec<(String, u32)> = sqlx::query_as(
            r#"
            SELECT *
            FROM white_monster_counter
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
}