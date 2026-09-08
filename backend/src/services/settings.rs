//! Typed access to the key/value `settings` table (system-wide configuration edited from the UI).

use sqlx::SqlitePool;

pub struct SettingsService {
    db: SqlitePool,
}

impl SettingsService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.db)
            .await?;
        Ok(row.map(|(v,)| v))
    }

    pub async fn get_or(&self, key: &str, default: &str) -> Result<String, sqlx::Error> {
        Ok(self.get(key).await?.unwrap_or_else(|| default.to_string()))
    }

    pub async fn get_bool(&self, key: &str, default: bool) -> Result<bool, sqlx::Error> {
        Ok(self
            .get(key)
            .await?
            .map(|v| matches!(v.as_str(), "true" | "1" | "yes"))
            .unwrap_or(default))
    }

    pub async fn get_u64(&self, key: &str, default: u64) -> Result<u64, sqlx::Error> {
        Ok(self.get(key).await?.and_then(|v| v.parse().ok()).unwrap_or(default))
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(&self.db)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::migrated_pool;

    #[tokio::test]
    async fn typed_get_set_and_prefix() {
        let svc = SettingsService::new(migrated_pool().await);
        assert_eq!(svc.get("x.a").await.unwrap(), None);
        assert!(svc.get_bool("x.flag", true).await.unwrap());
        svc.set("x.flag", "false").await.unwrap();
        svc.set("x.n", "42").await.unwrap();
        svc.set("x.n", "43").await.unwrap(); // upsert
        assert!(!svc.get_bool("x.flag", true).await.unwrap());
        assert_eq!(svc.get_u64("x.n", 0).await.unwrap(), 43);
        assert_eq!(svc.get_u64("x.missing", 7).await.unwrap(), 7);
    }
}
