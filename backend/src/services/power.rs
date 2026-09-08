//! Hardware & power: CPU frequency governor and scheduled reboot/shutdown.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::config::AppConfig;

const CPUFREQ: &str = "/sys/devices/system/cpu/cpu0/cpufreq";
const DAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

#[derive(Debug, Clone, Serialize)]
pub struct PowerStatus {
    pub governor: Option<String>,
    pub available_governors: Vec<String>,
    pub cpu_min_mhz: Option<u64>,
    pub cpu_max_mhz: Option<u64>,
    pub cpu_cur_mhz: Option<u64>,
    pub dev_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScheduledTask {
    pub id: String,
    pub kind: String,
    pub time: String,
    pub days: String,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledTaskInput {
    pub kind: String,
    pub time: String,
    pub days: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

pub struct PowerService {
    db: SqlitePool,
    dev_mode: bool,
}

impl PowerService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db, dev_mode: AppConfig::global().dev_mode }
    }

    async fn read_sys(name: &str) -> Option<String> {
        tokio::fs::read_to_string(format!("{}/{}", CPUFREQ, name)).await.ok().map(|s| s.trim().to_string())
    }

    pub async fn status(&self) -> PowerStatus {
        if self.dev_mode {
            let governor = crate::services::settings::SettingsService::new(self.db.clone())
                .get_or("power.governor", "ondemand")
                .await
                .unwrap_or_else(|_| "ondemand".into());
            return PowerStatus {
                governor: Some(governor),
                available_governors: vec!["conservative".into(), "ondemand".into(), "performance".into(), "powersave".into(), "schedutil".into()],
                cpu_min_mhz: Some(1500),
                cpu_max_mhz: Some(2400),
                cpu_cur_mhz: Some(1800),
                dev_mode: true,
            };
        }
        let khz = |v: Option<String>| v.and_then(|s| s.parse::<u64>().ok()).map(|k| k / 1000);
        PowerStatus {
            governor: Self::read_sys("scaling_governor").await,
            available_governors: Self::read_sys("scaling_available_governors")
                .await
                .map(|s| s.split_whitespace().map(|g| g.to_string()).collect())
                .unwrap_or_default(),
            cpu_min_mhz: khz(Self::read_sys("scaling_min_freq").await),
            cpu_max_mhz: khz(Self::read_sys("scaling_max_freq").await),
            cpu_cur_mhz: khz(Self::read_sys("scaling_cur_freq").await),
            dev_mode: false,
        }
    }

    pub async fn set_governor(&self, governor: &str) -> Result<()> {
        if !governor.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return Err(anyhow!("Invalid governor name"));
        }
        let status = self.status().await;
        if !status.available_governors.iter().any(|g| g == governor) {
            return Err(anyhow!("Governor '{}' is not available on this CPU", governor));
        }
        if self.dev_mode {
            crate::services::settings::SettingsService::new(self.db.clone()).set("power.governor", governor).await?;
            tracing::info!("[DEV MODE] Would set CPU governor to {}", governor);
            return Ok(());
        }
        let mut dirs = tokio::fs::read_dir("/sys/devices/system/cpu").await?;
        while let Some(entry) = dirs.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                let p = entry.path().join("cpufreq/scaling_governor");
                if tokio::fs::metadata(&p).await.is_ok() {
                    tokio::fs::write(&p, governor).await?;
                }
            }
        }
        // remember it for the next boot (applied by apply_saved_governor)
        crate::services::settings::SettingsService::new(self.db.clone()).set("power.governor", governor).await?;
        Ok(())
    }

    /// Re-apply the governor chosen in the UI (called at startup)
    pub async fn apply_saved_governor(&self) {
        if self.dev_mode {
            return;
        }
        if let Ok(Some(g)) = crate::services::settings::SettingsService::new(self.db.clone()).get("power.governor").await {
            if let Err(e) = self.set_governor(&g).await {
                tracing::warn!("Could not re-apply CPU governor {}: {}", g, e);
            }
        }
    }

    // ─── scheduled tasks ───────────────────────────────────────────

    fn validate(input: &ScheduledTaskInput) -> Result<(String, String)> {
        if !matches!(input.kind.as_str(), "reboot" | "shutdown") {
            return Err(anyhow!("kind must be 'reboot' or 'shutdown'"));
        }
        let parts: Vec<&str> = input.time.split(':').collect();
        let valid_time = parts.len() == 2
            && parts[0].parse::<u8>().map(|h| h < 24).unwrap_or(false)
            && parts[1].parse::<u8>().map(|m| m < 60).unwrap_or(false);
        if !valid_time {
            return Err(anyhow!("time must be HH:MM"));
        }
        let time = format!("{:02}:{:02}", parts[0].parse::<u8>().unwrap(), parts[1].parse::<u8>().unwrap());
        let mut days: Vec<&str> = Vec::new();
        for d in &input.days {
            let d = d.to_lowercase();
            match DAYS.iter().find(|x| **x == d) {
                Some(x) if !days.contains(x) => days.push(x),
                Some(_) => {}
                None => return Err(anyhow!("unknown day '{}'", d)),
            }
        }
        if days.is_empty() {
            return Err(anyhow!("at least one day is required"));
        }
        days.sort_by_key(|d| DAYS.iter().position(|x| x == d));
        Ok((time, days.join(",")))
    }

    pub async fn list_tasks(&self) -> Result<Vec<ScheduledTask>> {
        Ok(sqlx::query_as("SELECT * FROM scheduled_tasks ORDER BY time, kind").fetch_all(&self.db).await?)
    }

    pub async fn create_task(&self, input: ScheduledTaskInput) -> Result<ScheduledTask> {
        let (time, days) = Self::validate(&input)?;
        let task = ScheduledTask {
            id: uuid::Uuid::new_v4().to_string(),
            kind: input.kind,
            time,
            days,
            enabled: input.enabled,
            last_run_at: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        sqlx::query("INSERT INTO scheduled_tasks (id, kind, time, days, enabled, last_run_at, created_at) VALUES (?, ?, ?, ?, ?, NULL, ?)")
            .bind(&task.id).bind(&task.kind).bind(&task.time).bind(&task.days).bind(task.enabled).bind(&task.created_at)
            .execute(&self.db).await?;
        Ok(task)
    }

    pub async fn set_task_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        let r = sqlx::query("UPDATE scheduled_tasks SET enabled = ? WHERE id = ?").bind(enabled).bind(id).execute(&self.db).await?;
        if r.rows_affected() == 0 {
            return Err(anyhow!("Scheduled task not found"));
        }
        Ok(())
    }

    pub async fn delete_task(&self, id: &str) -> Result<()> {
        let r = sqlx::query("DELETE FROM scheduled_tasks WHERE id = ?").bind(id).execute(&self.db).await?;
        if r.rows_affected() == 0 {
            return Err(anyhow!("Scheduled task not found"));
        }
        Ok(())
    }

    /// Tasks whose time matches `now` (local time, HH:MM) and that did not already run this minute
    pub fn due<'a>(tasks: &'a [ScheduledTask], now_local: chrono::DateTime<chrono_tz::Tz>) -> Vec<&'a ScheduledTask> {
        let hhmm = now_local.format("%H:%M").to_string();
        let day = DAYS[now_local.format("%u").to_string().parse::<usize>().unwrap_or(1) - 1];
        let minute_key = now_local.format("%Y-%m-%dT%H:%M").to_string();
        tasks
            .iter()
            .filter(|t| t.enabled && t.time == hhmm && t.days.split(',').any(|d| d == day))
            .filter(|t| !t.last_run_at.as_deref().map(|l| l.starts_with(&minute_key)).unwrap_or(false))
            .collect()
    }

    /// Background scheduler: every 20s, run due tasks in the configured time zone
    pub fn start_scheduler(db: SqlitePool) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(std::time::Duration::from_secs(20));
            loop {
                ticker.tick().await;
                let svc = PowerService::new(db.clone());
                let tasks = match svc.list_tasks().await {
                    Ok(t) => t,
                    Err(e) => {
                        tracing::warn!("Scheduler: cannot list tasks: {}", e);
                        continue;
                    }
                };
                if tasks.iter().all(|t| !t.enabled) {
                    continue;
                }
                let tz_name = crate::services::timedate::TimeDateService::new(db.clone())
                    .status()
                    .await
                    .map(|s| s.timezone)
                    .unwrap_or_else(|_| "UTC".to_string());
                let tz: chrono_tz::Tz = tz_name.parse().unwrap_or(chrono_tz::UTC);
                let now_local = chrono::Utc::now().with_timezone(&tz);
                for task in Self::due(&tasks, now_local) {
                    let _ = sqlx::query("UPDATE scheduled_tasks SET last_run_at = ? WHERE id = ?")
                        .bind(now_local.format("%Y-%m-%dT%H:%M:%S%z").to_string())
                        .bind(&task.id)
                        .execute(&db)
                        .await;
                    tracing::warn!("Scheduled {} triggered (task {})", task.kind, task.id);
                    if svc.dev_mode {
                        tracing::info!("[DEV MODE] Would run: systemctl {}", task.kind);
                        continue;
                    }
                    let action = if task.kind == "reboot" { "reboot" } else { "poweroff" };
                    let _ = tokio::process::Command::new("systemctl").arg(action).status().await;
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::migrated_pool;
    use chrono::TimeZone;

    fn task(kind: &str, time: &str, days: &str, enabled: bool, last: Option<&str>) -> ScheduledTask {
        ScheduledTask { id: format!("{kind}-{time}"), kind: kind.into(), time: time.into(), days: days.into(), enabled, last_run_at: last.map(String::from), created_at: String::new() }
    }

    #[test]
    fn due_matches_time_day_and_skips_already_run() {
        let paris: chrono_tz::Tz = "Europe/Paris".parse().unwrap();
        let now = paris.with_ymd_and_hms(2026, 9, 8, 3, 30, 5).unwrap(); // a Tuesday
        let tasks = vec![
            task("reboot", "03:30", "mon,tue", true, None),
            task("shutdown", "03:30", "sat,sun", true, None),
            task("reboot", "03:31", "tue", true, None),
            task("shutdown", "03:30", "tue", false, None),
            task("reboot", "03:30", "tue", true, Some("2026-09-08T03:30:01+0200")),
        ];
        let due: Vec<&str> = PowerService::due(&tasks, now).iter().map(|t| t.id.as_str()).collect();
        assert_eq!(due, vec!["reboot-03:30"]);
    }

    #[tokio::test]
    async fn tasks_are_validated_and_persisted() {
        let svc = PowerService::new(migrated_pool().await);
        assert!(svc.create_task(ScheduledTaskInput { kind: "explode".into(), time: "03:00".into(), days: vec!["mon".into()], enabled: true }).await.is_err());
        assert!(svc.create_task(ScheduledTaskInput { kind: "reboot".into(), time: "25:00".into(), days: vec!["mon".into()], enabled: true }).await.is_err());
        assert!(svc.create_task(ScheduledTaskInput { kind: "reboot".into(), time: "03:00".into(), days: vec![], enabled: true }).await.is_err());
        let t = svc.create_task(ScheduledTaskInput { kind: "shutdown".into(), time: "3:5".into(), days: vec!["Sun".into(), "mon".into(), "mon".into()], enabled: true }).await.unwrap();
        assert_eq!(t.time, "03:05");
        assert_eq!(t.days, "mon,sun");
        svc.set_task_enabled(&t.id, false).await.unwrap();
        assert!(!svc.list_tasks().await.unwrap()[0].enabled);
        svc.delete_task(&t.id).await.unwrap();
        assert!(svc.delete_task(&t.id).await.is_err());
    }

    #[tokio::test]
    async fn dev_mode_governor_is_remembered() {
        let svc = PowerService { db: migrated_pool().await, dev_mode: true };
        assert!(svc.set_governor("warp-speed").await.is_err());
        svc.set_governor("performance").await.unwrap();
        assert_eq!(svc.status().await.governor.as_deref(), Some("performance"));
    }
}
