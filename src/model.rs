use std::{collections::HashMap, ops::Mul};

use rand::Rng;
use reqwest::Client;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use serde::Deserialize;
use tracing::{debug, trace, warn};

use crate::config::Config;

// pub enum CryptoError {
//     SymbolNotFound,
//     UnexpectedApiError,
// }

pub struct Model {
    config: Config,
    db_pool: SqlitePool,
    pub daily_amount: f64,
}

pub struct CoinInfo {
    pub symbol: String,
    pub name: String,
    pub current_price: f64,
}

#[derive(Debug, Deserialize)]
struct CmcApiResponse {
    status: CmcStatus,
    data: Option<HashMap<String, CmcCryptoData>>,
}

#[derive(Debug, Deserialize)]
struct CmcStatus {
    error_code: i32,
    error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CmcCryptoData {
    name: String,
    quote: HashMap<String, CmcQuoteData>,
}

#[derive(Debug, Deserialize)]
struct CmcQuoteData {
    price: f64,
}

impl Model {
    pub async fn new(config: Config) -> Self {
        // TODO: Handle errors properly
        
        // Connect to SQLite (creates data.db if it doesn't exist)
        let db_pool = SqlitePool::connect_with(
            SqliteConnectOptions::new()
            .filename("data.db")
            .create_if_missing(true)
        ).await.unwrap();

        // Enable foreign_keys in sqlite
        sqlx::query(r#"PRAGMA foreign_keys = ON;"#)
            .execute(&db_pool)
            .await
            .unwrap();

        // Create white monster counter table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS white_monster_counter (
                user_id TEXT NOT NULL PRIMARY KEY,
                count INT NOT NULL
            )
            "#,
        )
        .execute(&db_pool)
        .await
        .unwrap();

        // Create bank table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bank (
                user_id TEXT NOT NULL PRIMARY KEY,
                balance REAL NOT NULL,
                last_daily DATETIME NOT NULL
            )
            "#,
        )
        .execute(&db_pool)
        .await
        .unwrap();
        
        // Create portfolio table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS transactions (
                transaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                coin_symbol TEXT NOT NULL,
                amount REAL NOT NULL,
                price REAL NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES bank(user_id)
            )
            "#,
        )
        .execute(&db_pool)
        .await
        .unwrap();

        Self { 
            config,
            db_pool,
            daily_amount: 100f64,
        }
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
            INSERT INTO bank VALUES ($1, 0, DATE('now', '-1 day'))
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

    // Return crypto coin name and price
    pub async fn coin_info(&self, coin_symbol: &str) -> Option<CoinInfo> {
        let url = format!(
            "https://{}-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest?symbol={}&convert=eur",
            if self.config.use_cmc_sandbox_api { "sandbox" } else { "pro" },
            coin_symbol
        );

        let response = Client::new()
            .get(&url)
            .header("X-CMC_PRO_API_KEY", &self.config.cmc_api_key)
            .send()
            .await
            .ok()?
            .json::<CmcApiResponse>()
            .await
            .ok()?;

        if response.status.error_code != 0 {
            warn!("Cmc request returned with error_code {}", response.status.error_code);
        }

        let data = response.data?;        
        debug!("data: {:?}", data);

        let crypto_data = data.get(coin_symbol.to_uppercase().as_str())?;

        Some(CoinInfo {
            symbol: coin_symbol.into(),
            name: crypto_data.name.clone(),
            current_price: crypto_data.quote.get("EUR")?.price,
        })
    }

    pub async fn portfolio(&self, user_id: u64) -> Option<Vec<(String, f64, f64)>>{
        println!("Before getting query data");
        let mut portfolio_raw_data: Vec<(String, f64, f64)> = sqlx::query_as(
            r#"
            SELECT coin_symbol, SUM(amount) AS total_amount, 0. AS total_value
            FROM transactions
            WHERE user_id = $1
            GROUP BY coin_symbol
            HAVING total_amount > 0;
            "#
        )
        .bind(user_id.to_string())
        .fetch_all(&self.db_pool)
        .await
        .ok()?;

        for (coin_symbol, total_amount, total_value) in &mut portfolio_raw_data {
            let current_price = self.coin_info(&coin_symbol).await?.current_price;
            *total_value = total_amount.mul(current_price);
        }

        Some(portfolio_raw_data)
    }

    /// Create a transaction, returns amount of coins bought/sold and current price
    pub async fn buy(&self, user_id: u64, coin_symbol: &str, euro_amount: f64) -> Option<(f64, f64)> {
        // Check if amount is positive
        if euro_amount <= 0f64 {
            return None;
        }

        // Get current coin price. Indirectly also checks if symbol is valid
        let Some(coin_info) = self.coin_info(coin_symbol).await else {
            return None;
        };
        println!("Afer getting coin_info");

        // Convert euro_amount to coin_amount
        let coin_amount = euro_amount / coin_info.current_price;

        trace!("Creating transaction of {} {}", coin_amount, coin_symbol);
        let res = sqlx::query(
            r#"
            INSERT INTO transactions (user_id, coin_symbol, amount, price)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(user_id.to_string())
        .bind(coin_symbol.to_uppercase())
        .bind(coin_amount)
        .bind(coin_info.current_price)
        .execute(&self.db_pool)
        .await;

        // Remove euros_amount from balance and make this a db transaction
        self.bless(user_id, -euro_amount).await.unwrap();
        
        // FIXME: If we get foregn key constraint violation then return error user doesn have a bank account

        res.is_ok().then_some((coin_amount, coin_info.current_price))
    }

    pub async fn sell(&self, user_id: u64, coin_symbol: &str, euro_amount: f64) -> Option<(f64, f64)> {
        // Check if amount is positive (we are selling a positive ammount)
        if euro_amount <= 0f64 {
            return None;
        }

        // Get current coin price. Indirectly also checks if symbol is valid
        let Some(coin_info) = self.coin_info(coin_symbol).await else {
            return None;
        };

        // Convert user_id to string here for reuse.
        let user_id_str = user_id.to_string();

        // Convert euro_amount to coin_amount
        let coin_amount = euro_amount / coin_info.current_price;

        // Check if user_id has this amount of coins
        let owned_coin_amount: f64 = sqlx::query_scalar(
            r#"
            SELECT SUM(amount)
            FROM transactions
            WHERE user_id = $1 AND coin_symbol = $2;
            "#
        )
        .bind(user_id_str)
        .bind(coin_symbol)
        .fetch_one(&self.db_pool)
        .await
        .ok()?;

        // Only proceed with transaction if user has at least the amount of
        // coins intended to sell.
        if owned_coin_amount < coin_amount {
            return None;
        }

        // Create sell transaction
        trace!("Creating transaction of {} {}", coin_amount, coin_symbol);
        let res = sqlx::query(
            r#"
            INSERT INTO transactions (user_id, coin_symbol, amount, price)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(user_id.to_string())
        .bind(coin_symbol)
        .bind(-1f64 * coin_amount)
        .bind(coin_info.current_price)
        .execute(&self.db_pool)
        .await;

        // Add euros_amount to balance and make this a db transaction
        self.bless(user_id, euro_amount).await.unwrap();
        
        // FIXME: If we get foreign key constraint violation then return error user doesn have a bank account

        res.is_ok().then_some((coin_amount, coin_info.current_price))
    }

    pub async fn sell_all(&self, user_id: u64, coin_symbol: &str) -> Option<(f64, f64)> {
        // Get current coin price. Indirectly also checks if symbol is valid
        let Some(coin_info) = self.coin_info(coin_symbol).await else {
            return None;
        };

        // Convert user_id to string here for reuse.
        let user_id_str = user_id.to_string();
        let coin_symbol = coin_symbol.to_uppercase();
        // Get amount of coins owned by this user_id
        let owned_coin_amount: f64 = sqlx::query_scalar(
            r#"
            SELECT SUM(amount)
            FROM transactions
            WHERE user_id = $1 AND coin_symbol = $2;
            "#
        )
        .bind(user_id_str)
        .bind(&coin_symbol)
        .fetch_one(&self.db_pool)
        .await
        .ok()?;

        // Only proceed with transaction if user has positive amount of
        // coins intended to sell.
        if owned_coin_amount <= 0f64 {
            return None;
        }

        // Create sell transaction
        trace!("Creating transaction of {} {}", owned_coin_amount, coin_symbol);
        let res = sqlx::query(
            r#"
            INSERT INTO transactions (user_id, coin_symbol, amount, price)
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(user_id.to_string())
        .bind(&coin_symbol)
        .bind(-1f64 * owned_coin_amount)
        .bind(coin_info.current_price)
        .execute(&self.db_pool)
        .await;
        
        // Add euros_amount to balance and make this a db transaction
        self.bless(user_id, owned_coin_amount * coin_info.current_price).await.unwrap();
        
        // FIXME: If we get foreign key constraint violation then return error user doesn have a bank account

        res.is_ok().then_some((owned_coin_amount, coin_info.current_price))
    }

    pub async fn coin_flip(&self, user_id: u64, choice: &str, bet: f64) -> Option<bool> {
        // Bet must be positive amount
        if bet <= 0f64 {
            return None;
        }

        // Flip coin with 50/50 probability
        let flip_result = rand::rng().random_bool(0.5);
        
        // true is heads, false is tails 
        let has_won = match choice {
            "heads" => { flip_result },
            "tails" => { !flip_result },
            _ => { return None; },
        };

        // Update user funds, add bet if won, subtract bet otherwise
        sqlx::query(
            r#"
            UPDATE bank
            SET balance = balance + $2
            WHERE user_id = $1 AND balance + $2 >= 0
            "#
        )
        .bind(user_id.to_string())
        .bind(if has_won { bet } else {-bet})
        .execute(&self.db_pool)
        .await
        .ok()?;
    
        Some(has_won)
    }

    pub async fn daily(&self, user_id: u64) -> Option<bool> {
        // Used to check if user account exists
        self.balance(user_id).await?;
        
        let res = sqlx::query(
            r#"
            UPDATE bank
            SET last_daily = CURRENT_TIMESTAMP,
                balance = balance + $2
            WHERE user_id = $1
            AND DATE(last_daily) <> DATE(CURRENT_TIMESTAMP)
            "#
        )
        .bind(user_id.to_string())
        .bind(self.daily_amount)
        .execute(&self.db_pool)
        .await
        .ok()?;

        Some(res.rows_affected() > 0)
    }
}