use std::{collections::HashMap, ops::Mul};

use poise::serenity_prelude::model::permissions;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing::{trace, warn};

use crate::{config::Config, permissions::Permission};

/// Model errors are errors that will be sent to the user.
#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error("Invalid value, {0}")]
    InvalidValue(String),
    #[error("User {0} does not have bank account.")]
    BankAccountNotFound(u64),
    #[error("Insuficient funds.")]
    InsuficientFunds,
    #[error("Insuficient coins.")]
    InsuficientCoins,
    #[error("Unexpected error.")]
    UnexpectedError,
}

impl From<sqlx::Error> for ModelError {
    fn from(_: sqlx::Error) -> Self {
        ModelError::UnexpectedError
    }
}

type ModelResult<T> = std::result::Result<T, ModelError>;

pub struct Model {
    config: Config,
    db_pool: SqlitePool,
    permissions: HashMap<u64, Permission>,
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
    pub async fn new(config: Config) -> ModelResult<Self> {
        // TODO: Handle errors properly

        // Connect to SQLite (creates data.db if it doesn't exist)
        let db_pool = SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename("data.db")
                .create_if_missing(true),
        )
        .await?;

        // Enable foreign_keys in sqlite
        sqlx::query(r#"PRAGMA foreign_keys = ON;"#)
            .execute(&db_pool)
            .await?;

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
        .await?;

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
        .await?;

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
        .await?;

        let mut permissions = HashMap::new();
        permissions.insert(181002804813496320u64, Permission::Admin);

        Ok(Self {
            config,
            db_pool,
            permissions,
            daily_amount: 100f64,
        })
    }

    pub fn user_has_permission(&self, user_id: u64, permission: Permission) -> bool {
        println!("{:?}", self.permissions);
        println!("{:?}", self.permissions.get(&user_id));
        println!(
            "{:?}",
            self.permissions
                .get(&user_id)
                .is_some_and(|p| *p < permission)
        );

        self.permissions
            .get(&user_id)
            .is_some_and(|p| *p >= permission)
    }

    pub async fn wm_counters(&self) -> (u32, Vec<(u64, u32)>) {
        // FIXME: Handle results properly
        let wm_data: Vec<(String, u32)> = sqlx::query_as(
            r#"
            SELECT user_id, count
            FROM white_monster_counter
            ORDER BY count DESC
            "#,
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

    pub async fn inc_wm_counter(&self, user_id: u64, amount: u32) -> ModelResult<u32> {
        let wm_counter: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO white_monster_counter(user_id, count)
            VALUES($1, $2) ON CONFLICT(user_id) DO
            UPDATE SET count = count + $2
            RETURNING count;
            "#,
        )
        .bind(user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(u32::try_from(wm_counter).map_err(|_| ModelError::UnexpectedError)?)
    }

    pub async fn dec_wm_counter(&self, user_id: u64, amount: u32) -> ModelResult<u32> {
        // FIXME: Handle results properly
        let wm_counter: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO white_monster_counter(user_id, count)
            VALUES($1, $2) ON CONFLICT(user_id) DO
            UPDATE SET count = count - $2
            RETURNING count;
            "#,
        )
        .bind(user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(u32::try_from(wm_counter).map_err(|_| ModelError::UnexpectedError)?)
    }

    pub async fn balance(&self, user_id: u64) -> ModelResult<f64> {
        let res = sqlx::query_scalar(
            r#"
            SELECT balance FROM bank WHERE user_id = $1
            "#,
        )
        .bind(user_id.to_string())
        .fetch_one(&self.db_pool)
        .await;

        match res {
            Ok(balance) => Ok(balance),
            Err(sqlx::Error::RowNotFound) => Err(ModelError::BankAccountNotFound(user_id)),
            Err(_) => Err(ModelError::UnexpectedError),
        }
    }

    pub async fn create_bank_account(&self, user_id: u64) -> ModelResult<()> {
        // By default user starts with 0 euros.
        // If there's already a bank account for this user_id this query will
        // fail, but we won't use the error so we don't care and just throw
        // UnexpectedError.
        sqlx::query(
            r#"
            INSERT INTO bank VALUES ($1, 0, DATE('now', '-1 day'))
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    // Returns ModelError::InsuficientFunds if src_user has insuficient funds
    pub async fn give(
        &self,
        src_user_id: u64,
        dst_user_id: u64,
        amount: f64,
    ) -> ModelResult<(f64, f64)> {
        // Check if src_user_id and dst_user_id account exists.
        self.balance(src_user_id).await?;
        self.balance(dst_user_id).await?;

        if amount <= 0f64 {
            return Err(ModelError::InvalidValue(
                "'amount' must be positive.".into(),
            ));
        }

        // Update src_user balance.
        let src_new_balance_res = sqlx::query_scalar(
            r#"
            UPDATE bank SET balance = balance - $2
            WHERE user_id = $1 AND balance >= $2
            RETURNING balance
            "#,
        )
        .bind(src_user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await;

        let src_new_balance = match src_new_balance_res {
            Ok(Some(src_new_balance)) => src_new_balance,
            Err(sqlx::Error::RowNotFound) => {
                return Err(ModelError::InsuficientFunds);
            }
            _ => {
                return Err(ModelError::UnexpectedError);
            }
        };

        // Update dst_user balance.
        let dst_new_balance: f64 = sqlx::query_scalar(
            r#"
            UPDATE bank SET balance = balance + $2
            WHERE user_id = $1
            RETURNING balance
            "#,
        )
        .bind(dst_user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await?;

        Ok((src_new_balance, dst_new_balance))
    }

    pub async fn bless(&self, dst_user_id: u64, amount: f64) -> ModelResult<f64> {
        let new_balance_res = sqlx::query_scalar(
            r#"
            UPDATE bank SET balance = MAX(balance + $2, 0)
            WHERE user_id = $1
            RETURNING balance
            "#,
        )
        .bind(dst_user_id.to_string())
        .bind(amount)
        .fetch_one(&self.db_pool)
        .await;

        match new_balance_res {
            Ok(new_balance) => Ok(new_balance),
            Err(sqlx::Error::RowNotFound) => Err(ModelError::BankAccountNotFound(dst_user_id)),
            Err(_) => Err(ModelError::UnexpectedError),
        }
    }

    pub async fn leaderboard(&self) -> ModelResult<Vec<(u64, f64)>> {
        let bank_data: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT user_id, balance
            FROM bank
            ORDER BY balance DESC
            "#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        let bank_data_converted = bank_data
            .iter()
            .map(|(user_id, count)| (user_id.parse::<u64>().unwrap(), *count))
            .collect::<Vec<(u64, f64)>>();
        Ok(bank_data_converted)
    }

    // Return crypto coin name and price
    pub async fn coin_info(&self, coin_symbol: &str) -> ModelResult<CoinInfo> {
        let url = format!(
            "https://{}-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest?symbol={}&convert=eur",
            if self.config.use_cmc_sandbox_api {
                "sandbox"
            } else {
                "pro"
            },
            coin_symbol
        );

        let response = Client::new()
            .get(&url)
            .header("X-CMC_PRO_API_KEY", &self.config.cmc_api_key)
            .send()
            .await
            .map_err(|_| ModelError::UnexpectedError)?
            .json::<CmcApiResponse>()
            .await
            .map_err(|_| ModelError::UnexpectedError)?;

        if response.status.error_code != 0 {
            warn!(
                "Cmc request returned with error_code {}",
                response.status.error_code
            );
        }

        let data = response.data.ok_or(ModelError::UnexpectedError)?;

        let crypto_data = data
            .get(coin_symbol.to_uppercase().as_str())
            .ok_or(ModelError::InvalidValue("'symbol' does not exist.".into()))?;

        Ok(CoinInfo {
            symbol: coin_symbol.into(),
            name: crypto_data.name.clone(),
            current_price: crypto_data
                .quote
                .get("EUR")
                .ok_or(ModelError::UnexpectedError)?
                .price,
        })
    }

    pub async fn portfolio(&self, user_id: u64) -> ModelResult<Vec<(String, f64, f64)>> {
        // Check if user account exists
        self.balance(user_id).await?;

        let mut portfolio_raw_data: Vec<(String, f64, f64)> = sqlx::query_as(
            r#"
            SELECT coin_symbol, SUM(amount) AS total_amount, 0. AS total_value
            FROM transactions
            WHERE user_id = $1
            GROUP BY coin_symbol
            HAVING total_amount > 0;
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.db_pool)
        .await?;

        for (coin_symbol, total_amount, total_value) in &mut portfolio_raw_data {
            let current_price = self.coin_info(&coin_symbol).await?.current_price;
            *total_value = total_amount.mul(current_price);
        }

        Ok(portfolio_raw_data)
    }

    /// Create a transaction, returns amount of coins bought/sold and current price
    pub async fn buy(
        &self,
        user_id: u64,
        coin_symbol: &str,
        euro_amount: f64,
    ) -> ModelResult<(f64, f64)> {
        // Check if amount is positive
        if euro_amount <= 0f64 {
            return Err(ModelError::InvalidValue(
                "'amount' must be positive.".into(),
            ));
        }

        // Get current coin price. Indirectly also checks if symbol is valid.
        let coin_info = self.coin_info(coin_symbol).await?;

        // Convert user_id to string here for reuse.
        let user_id_str = user_id.to_string();

        // Convert euro_amount to coin_amount.
        let coin_amount = euro_amount / coin_info.current_price;

        // Check if user has bank account and suficient balance.
        let balance = self.balance(user_id).await?;
        if balance < coin_amount {
            return Err(ModelError::InsuficientFunds);
        }

        trace!("Creating transaction of {} {}", coin_amount, coin_symbol);
        sqlx::query(
            r#"
            INSERT INTO transactions (user_id, coin_symbol, amount, price)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(&user_id_str)
        .bind(coin_symbol.to_uppercase())
        .bind(coin_amount)
        .bind(coin_info.current_price)
        .execute(&self.db_pool)
        .await?;

        // Remove euros_amount from balance and make this a db transaction.
        self.bless(user_id, -euro_amount).await?;

        // FIXME: If we get foregn key constraint violation then return error user doesn have a bank account

        Ok((coin_amount, coin_info.current_price))
    }

    pub async fn sell(
        &self,
        user_id: u64,
        coin_symbol: &str,
        euro_amount: f64,
    ) -> ModelResult<(f64, f64)> {
        // Check if user account exists
        self.balance(user_id).await?;

        // Check if amount is positive (we are selling a positive ammount)
        if euro_amount <= 0f64 {
            return Err(ModelError::InvalidValue(
                "'amount' must be positive.".into(),
            ));
        }

        // Get current coin price. Indirectly also checks if symbol is valid
        let coin_info = self.coin_info(coin_symbol).await?;

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
            "#,
        )
        .bind(user_id_str)
        .bind(coin_symbol)
        .fetch_one(&self.db_pool)
        .await?;

        // Only proceed with transaction if user has at least the amount of
        // coins intended to sell.
        if owned_coin_amount < coin_amount {
            return Err(ModelError::InsuficientCoins);
        }

        // Create sell transaction
        trace!(
            "Creating transaction of {} {}",
            -1f64 * coin_amount,
            coin_symbol
        );
        sqlx::query(
            r#"
            INSERT INTO transactions (user_id, coin_symbol, amount, price)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id.to_string())
        .bind(coin_symbol)
        .bind(-1f64 * coin_amount)
        .bind(coin_info.current_price)
        .execute(&self.db_pool)
        .await?;

        // Add euros_amount to balance and make this a db transaction
        self.bless(user_id, euro_amount).await?;

        // FIXME: If we get foreign key constraint violation then return error user doesn have a bank account

        Ok((coin_amount, coin_info.current_price))
    }

    pub async fn sell_all(&self, user_id: u64, coin_symbol: &str) -> ModelResult<(f64, f64)> {
        // Check if user account exists
        self.balance(user_id).await?;

        // Get current coin price. Indirectly also checks if symbol is valid
        let coin_info = self.coin_info(coin_symbol).await?;

        // Convert user_id to string here for reuse.
        let user_id_str = user_id.to_string();
        let coin_symbol = coin_symbol.to_uppercase();
        // Get amount of coins owned by this user_id
        let owned_coin_amount: f64 = sqlx::query_scalar(
            r#"
            SELECT SUM(amount)
            FROM transactions
            WHERE user_id = $1 AND coin_symbol = $2;
            "#,
        )
        .bind(user_id_str)
        .bind(&coin_symbol)
        .fetch_one(&self.db_pool)
        .await?;

        // Only proceed with transaction if user has positive amount of
        // coins intended to sell.
        if owned_coin_amount <= 0f64 {
            return Err(ModelError::InsuficientCoins);
        }

        // Create sell transaction
        trace!(
            "Creating transaction of {} {}",
            -1f64 * owned_coin_amount,
            coin_symbol
        );
        sqlx::query(
            r#"
            INSERT INTO transactions (user_id, coin_symbol, amount, price)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(user_id.to_string())
        .bind(&coin_symbol)
        .bind(-1f64 * owned_coin_amount)
        .bind(coin_info.current_price)
        .execute(&self.db_pool)
        .await?;

        // Add euros_amount to balance and make this a db transaction
        self.bless(user_id, owned_coin_amount * coin_info.current_price)
            .await?;

        // FIXME: If we get foreign key constraint violation then return error user doesn have a bank account

        Ok((owned_coin_amount, coin_info.current_price))
    }

    pub async fn coin_flip(&self, user_id: u64, choice: &str, bet: f64) -> ModelResult<bool> {
        // Bet must be positive amount
        if bet <= 0f64 {
            return Err(ModelError::InvalidValue(
                "'amount' must be positive.".into(),
            ));
        }

        // Flip coin with 50/50 probability
        let flip_result = rand::rng().random_bool(0.5);

        // true is heads, false is tails
        let has_won = match choice {
            "heads" => flip_result,
            "tails" => !flip_result,
            _ => {
                return Err(ModelError::InvalidValue(
                    "'choice' must be 'heads' or 'tails'.".into(),
                ));
            }
        };

        // Update user funds, add bet if won, subtract bet otherwise
        sqlx::query(
            r#"
            UPDATE bank
            SET balance = balance + $2
            WHERE user_id = $1 AND balance + $2 >= 0
            "#,
        )
        .bind(user_id.to_string())
        .bind(if has_won { bet } else { -bet })
        .execute(&self.db_pool)
        .await?;

        Ok(has_won)
    }

    pub async fn daily(&self, user_id: u64) -> ModelResult<bool> {
        // Check if user account exists
        self.balance(user_id).await?;

        let res = sqlx::query(
            r#"
            UPDATE bank
            SET last_daily = CURRENT_TIMESTAMP,
                balance = balance + $2
            WHERE user_id = $1
            AND DATE(last_daily) <> DATE(CURRENT_TIMESTAMP)
            "#,
        )
        .bind(user_id.to_string())
        .bind(self.daily_amount)
        .execute(&self.db_pool)
        .await?;

        Ok(res.rows_affected() > 0)
    }
}
