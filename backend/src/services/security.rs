//! Security settings: login audit, session lifetime, TLS switch.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::config::AppConfig;
use crate::services::settings::SettingsService;

pub const DEFAULT_SESSION_HOURS: u64 = 24;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct LoginAttempt {
    pub id: String,
    pub username: String,
    pub ip: String,
    pub success: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    /// Lifetime of a login session (JWT + cookie), in hours
    pub session_hours: u64,
    /// Desired TLS state (applied at next service restart)
    pub tls_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SecurityStatus {
    pub settings: SecuritySettings,
    /// TLS state of the running process
    pub tls_active: bool,
    /// True when `settings.tls_enabled` differs from `tls_active`
    pub restart_required: bool,
    pub tls_cert_present: bool,
    pub tls_cert_generated_at: Option<String>,
    pub failed_logins_24h: i64,
    pub dev_mode: bool,
}

pub struct SecurityService {
    db: SqlitePool,
}

impl SecurityService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    pub async fn record_login(&self, username: &str, ip: &str, success: bool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO login_attempts (id, username, ip, success, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(username)
            .bind(ip)
            .bind(success)
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&self.db)
            .await?;
        // keep the table bounded
        sqlx::query("DELETE FROM login_attempts WHERE id NOT IN (SELECT id FROM login_attempts ORDER BY created_at DESC LIMIT 5000)")
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn login_attempts(&self, limit: i64) -> Result<Vec<LoginAttempt>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM login_attempts ORDER BY created_at DESC LIMIT ?")
            .bind(limit)
            .fetch_all(&self.db)
            .await
    }

    pub async fn failed_logins_since(&self, since: chrono::DateTime<chrono::Utc>) -> Result<i64, sqlx::Error> {
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM login_attempts WHERE success = FALSE AND created_at >= ?")
            .bind(since.to_rfc3339())
            .fetch_one(&self.db)
            .await?;
        Ok(n)
    }

    pub async fn settings(&self) -> Result<SecuritySettings, sqlx::Error> {
        let s = SettingsService::new(self.db.clone());
        let cfg = AppConfig::global();
        Ok(SecuritySettings {
            session_hours: s.get_u64("security.session_hours", cfg.jwt_expiration_hours).await?.clamp(1, 24 * 30),
            tls_enabled: s.get_bool("security.tls_enabled", cfg.tls_enabled).await?,
        })
    }

    /// Session lifetime to use for a new login
    pub async fn session_hours(db: &SqlitePool) -> u64 {
        SettingsService::new(db.clone())
            .get_u64("security.session_hours", AppConfig::global().jwt_expiration_hours)
            .await
            .unwrap_or(DEFAULT_SESSION_HOURS)
            .clamp(1, 24 * 30)
    }

    /// Desired TLS state, or None when never configured from the UI
    pub async fn tls_setting(db: &SqlitePool) -> Option<bool> {
        SettingsService::new(db.clone())
            .get("security.tls_enabled")
            .await
            .ok()
            .flatten()
            .map(|v| matches!(v.as_str(), "true" | "1"))
    }

    pub async fn save(&self, settings: &SecuritySettings) -> Result<(), sqlx::Error> {
        let s = SettingsService::new(self.db.clone());
        s.set("security.session_hours", &settings.session_hours.clamp(1, 24 * 30).to_string()).await?;
        s.set("security.tls_enabled", &settings.tls_enabled.to_string()).await?;
        Ok(())
    }

    pub async fn status(&self, tls_active: bool) -> Result<SecurityStatus, sqlx::Error> {
        let cfg = AppConfig::global();
        let settings = self.settings().await?;
        let cert_path = std::path::Path::new(&cfg.data_dir()).join(".tls").join("cert.pem");
        let cert_meta = tokio::fs::metadata(&cert_path).await.ok();
        let generated_at = cert_meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());
        Ok(SecurityStatus {
            restart_required: settings.tls_enabled != tls_active,
            settings,
            tls_active,
            tls_cert_present: cert_meta.is_some(),
            tls_cert_generated_at: generated_at,
            failed_logins_24h: self.failed_logins_since(chrono::Utc::now() - chrono::Duration::hours(24)).await?,
            dev_mode: cfg.dev_mode,
        })
    }

    /// Delete the self-signed certificate so a fresh one is generated at next start
    pub async fn reset_tls_certificate(&self) -> std::io::Result<()> {
        let tls_dir = std::path::Path::new(&AppConfig::global().data_dir()).join(".tls");
        for f in ["cert.pem", "key.pem"] {
            let p = tls_dir.join(f);
            if tokio::fs::metadata(&p).await.is_ok() {
                tokio::fs::remove_file(&p).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::migrated_pool;

    #[tokio::test]
    async fn login_audit_and_settings_round_trip() {
        let pool = migrated_pool().await;
        let svc = SecurityService::new(pool.clone());
        svc.record_login("alice", "10.0.0.2", false).await.unwrap();
        svc.record_login("alice", "10.0.0.2", true).await.unwrap();
        let attempts = svc.login_attempts(10).await.unwrap();
        assert_eq!(attempts.len(), 2);
        assert!(attempts[0].success, "newest first");
        assert_eq!(svc.failed_logins_since(chrono::Utc::now() - chrono::Duration::hours(1)).await.unwrap(), 1);

        let before = svc.settings().await.unwrap();
        assert_eq!(before.session_hours, AppConfig::global().jwt_expiration_hours);
        svc.save(&SecuritySettings { session_hours: 72, tls_enabled: true }).await.unwrap();
        let after = svc.settings().await.unwrap();
        assert_eq!(after.session_hours, 72);
        assert!(after.tls_enabled);
        assert_eq!(SecurityService::session_hours(&pool).await, 72);
        assert_eq!(SecurityService::tls_setting(&pool).await, Some(true));
        let status = svc.status(false).await.unwrap();
        assert!(status.restart_required);
        assert_eq!(status.failed_logins_24h, 1);
    }
}
