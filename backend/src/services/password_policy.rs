//! Password policy stored in `settings` (password.*) and enforced on every password change.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::services::settings::SettingsService;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasswordPolicy {
    pub min_length: u64,
    pub require_upper_lower: bool,
    pub require_number: bool,
    pub require_special: bool,
    /// Reject passwords containing the username
    pub forbid_username: bool,
    /// 0 = never expires
    pub expiry_days: u64,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_upper_lower: false,
            require_number: false,
            require_special: false,
            forbid_username: true,
            expiry_days: 0,
        }
    }
}

impl PasswordPolicy {
    pub async fn load(db: &SqlitePool) -> Result<Self, sqlx::Error> {
        let s = SettingsService::new(db.clone());
        let d = Self::default();
        Ok(Self {
            min_length: s.get_u64("password.min_length", d.min_length).await?.clamp(4, 128),
            require_upper_lower: s.get_bool("password.require_upper_lower", d.require_upper_lower).await?,
            require_number: s.get_bool("password.require_number", d.require_number).await?,
            require_special: s.get_bool("password.require_special", d.require_special).await?,
            forbid_username: s.get_bool("password.forbid_username", d.forbid_username).await?,
            expiry_days: s.get_u64("password.expiry_days", d.expiry_days).await?,
        })
    }

    pub async fn save(&self, db: &SqlitePool) -> Result<(), sqlx::Error> {
        let s = SettingsService::new(db.clone());
        s.set("password.min_length", &self.min_length.clamp(4, 128).to_string()).await?;
        s.set("password.require_upper_lower", &self.require_upper_lower.to_string()).await?;
        s.set("password.require_number", &self.require_number.to_string()).await?;
        s.set("password.require_special", &self.require_special.to_string()).await?;
        s.set("password.forbid_username", &self.forbid_username.to_string()).await?;
        s.set("password.expiry_days", &self.expiry_days.to_string()).await?;
        Ok(())
    }

    /// Human-readable reason the password is refused, or `None` when it complies
    pub fn violation(&self, password: &str, username: Option<&str>) -> Option<String> {
        if password.chars().count() < self.min_length as usize {
            return Some(format!("Password must be at least {} characters", self.min_length));
        }
        if self.require_upper_lower
            && !(password.chars().any(|c| c.is_uppercase()) && password.chars().any(|c| c.is_lowercase()))
        {
            return Some("Password must contain upper and lower case letters".to_string());
        }
        if self.require_number && !password.chars().any(|c| c.is_ascii_digit()) {
            return Some("Password must contain a number".to_string());
        }
        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Some("Password must contain a special character".to_string());
        }
        if self.forbid_username {
            if let Some(u) = username.filter(|u| u.len() >= 3) {
                if password.to_lowercase().contains(&u.to_lowercase()) {
                    return Some("Password must not contain the username".to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::migrated_pool;

    #[test]
    fn default_policy_checks_length_and_username() {
        let p = PasswordPolicy::default();
        assert!(p.violation("short", None).is_some());
        assert!(p.violation("longenough", None).is_none());
        assert!(p.violation("alice-2026", Some("alice")).is_some());
        assert!(p.violation("alice-2026", Some("bob")).is_none());
    }

    #[test]
    fn strict_policy_checks_classes() {
        let p = PasswordPolicy { min_length: 10, require_upper_lower: true, require_number: true, require_special: true, ..Default::default() };
        assert!(p.violation("alllowercase1!", None).is_some());
        assert!(p.violation("NoDigitsHere!", None).is_some());
        assert!(p.violation("NoSpecial123", None).is_some());
        assert!(p.violation("Str0ng-Pass!", None).is_none());
    }

    #[tokio::test]
    async fn policy_round_trips_through_settings() {
        let pool = migrated_pool().await;
        assert_eq!(PasswordPolicy::load(&pool).await.unwrap(), PasswordPolicy::default());
        let custom = PasswordPolicy { min_length: 12, require_number: true, expiry_days: 90, ..Default::default() };
        custom.save(&pool).await.unwrap();
        assert_eq!(PasswordPolicy::load(&pool).await.unwrap(), custom);
    }
}
