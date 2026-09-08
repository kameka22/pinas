use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A persisted, user-facing notification (storage alert, app install, update, security…)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: String,
    /// info | success | warning | error
    pub level: String,
    pub title: String,
    pub message: String,
    /// Producer: storage | app-center | update | auth | system
    pub source: String,
    pub read: bool,
    pub created_at: String,
    /// Optional key that collapses repeats of the same condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedupe_key: Option<String>,
}
