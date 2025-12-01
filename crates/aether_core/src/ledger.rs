use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use anyhow::Result;
use std::str::FromStr;

pub struct LedgerManager {
    pool: SqlitePool,
}

impl LedgerManager {
    pub async fn new(db_path: &str, key: &str) -> Result<Self> {
        // In a real implementation, we would use the key for SQLCipher encryption.
        // For now, we use standard SQLite for the skeleton.
        let options = SqliteConnectOptions::from_str(db_path)?
            .create_if_missing(true);
            // .pragma("key", key); // SQLCipher specific

        let pool = SqlitePool::connect_with(options).await?;
        
        // Initialize schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                sender TEXT NOT NULL,
                recipient TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                status TEXT NOT NULL
            )"
        ).execute(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn log_success(&self, sender: &str, recipient: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();
        sqlx::query(
            "INSERT INTO transactions (sender, recipient, timestamp, status) VALUES (?, ?, ?, 'SUCCESS')"
        )
        .bind(sender)
        .bind(recipient)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
