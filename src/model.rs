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

    pub async fn wm_counter(&self) -> u32 {
        // FIXME: Handle results properly
        let wm_counter: i64 = sqlx::query_scalar(
            r#"
            SELECT counter
            FROM white_monster_counter
            "#
        )
        .fetch_one(&self.db_pool)
        .await
        .unwrap();

        u32::try_from(wm_counter).unwrap()
    }

    pub async fn inc_wm_counter(&self) -> u32 {
        // FIXME: Handle results properly
        let wm_counter: i64 = sqlx::query_scalar(
            r#"
            UPDATE white_monster_counter
            SET counter = counter + 1
            RETURNING counter;
            "#
        )
        .fetch_one(&self.db_pool)
        .await
        .unwrap();

        u32::try_from(wm_counter).unwrap()
    }

    pub async fn dec_wm_counter(&self) -> u32 {
        // FIXME: Handle results properly
        let wm_counter: i64 = sqlx::query_scalar(
            r#"
            UPDATE white_monster_counter
            SET counter = MAX(counter - 1, 0)
            RETURNING counter;
            "#
        )
        .fetch_one(&self.db_pool)
        .await
        .unwrap();

        u32::try_from(wm_counter).unwrap()
    }
}