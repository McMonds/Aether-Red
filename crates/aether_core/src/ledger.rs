use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use anyhow::Result;
use std::str::FromStr;
use chrono::{Utc, Duration};

pub struct LedgerManager {
    pool: SqlitePool,
}

impl LedgerManager {
    pub async fn new(db_path: &str, _key: &str) -> Result<Self> {
        // In a real implementation, we would use the key for SQLCipher encryption.
        // For now, we use standard SQLite for the skeleton.
        let options = SqliteConnectOptions::from_str(db_path)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal); // WAL Mode for performance

        let pool = SqlitePool::connect_with(options).await?;
        
        // Initialize schema
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                sender TEXT NOT NULL,
                recipient TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                status TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_recipient ON transactions(recipient);
            CREATE INDEX IF NOT EXISTS idx_sender_time ON transactions(sender, timestamp);"
        ).execute(&pool).await?;

        Ok(Self { pool })
    }

    /// ATOMIC LOGGING: Record a successful transaction
    pub async fn log_success(&self, sender: &str, recipient: &str) -> Result<()> {
        let timestamp = Utc::now().timestamp();
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

    /// DUPLICATE CHECK: Has this recipient already received an email?
    pub async fn is_already_sent(&self, recipient: &str) -> Result<bool> {
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM transactions WHERE recipient = ? AND status = 'SUCCESS')"
        )
        .bind(recipient)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.0)
    }

    /// 24-HOUR ROLLING LIMIT: Calculate remaining capacity for a sender
    pub async fn get_sender_capacity(&self, sender: &str, max_daily: i64) -> Result<i64> {
        let cutoff = (Utc::now() - Duration::hours(24)).timestamp();
        
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM transactions WHERE sender = ? AND timestamp > ?"
        )
        .bind(sender)
        .bind(cutoff)
        .fetch_one(&self.pool)
        .await?;

        let used = row.0;
        let remaining = max_daily - used;
        
        Ok(if remaining < 0 { 0 } else { remaining })
    }
}
