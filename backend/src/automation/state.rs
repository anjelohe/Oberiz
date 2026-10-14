//! Automation's own progress/concurrency tracking, and the small
//! `automation_state` / `series_target_state` bookkeeping writes shared by
//! every search path (RSS grabs, scheduled cycles, manual runs).
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::RwLock;

use crate::AppState;

#[derive(Clone, Default)]
pub struct AutomationRuntime {
    pub(super) running: Arc<AtomicBool>,
    pub(super) progress: Arc<RwLock<AutomationProgress>>,
}

#[derive(Clone, Default)]
pub(super) struct AutomationProgress {
    pub(super) current_item: Option<String>,
    pub(super) completed_items: usize,
    pub(super) total_items: usize,
}

impl AutomationRuntime {
    /// Returns a guard that clears the running flag on `Drop` — including when
    /// the caller's future is cancelled mid-cycle rather than finishing
    /// normally. `run_now` used to call `finish()` as a plain statement after
    /// awaiting the cycle, but an HTTP handler's future is dropped outright if
    /// the client disconnects mid-request, which skipped that call and left
    /// `running` stuck at true forever — blocking every future manual run and
    /// the scheduled cycle until the process restarted.
    pub(super) fn begin(&self) -> Option<AutomationRunGuard> {
        self.running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
            .then(|| AutomationRunGuard {
                running: self.running.clone(),
            })
    }
    pub(super) async fn reset(&self, total: usize) {
        *self.progress.write().await = AutomationProgress {
            current_item: None,
            completed_items: 0,
            total_items: total,
        };
    }
    pub(super) async fn item(&self, label: String) {
        self.progress.write().await.current_item = Some(label);
    }
    pub(super) async fn done(&self) {
        let mut value = self.progress.write().await;
        value.completed_items += 1;
        value.current_item = None;
    }
}

pub(super) struct AutomationRunGuard {
    running: Arc<AtomicBool>,
}
impl Drop for AutomationRunGuard {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

pub(super) async fn record_series_target_error(
    state: &AppState,
    series_id: i64,
    season_number: i32,
    episode_number: Option<i32>,
    error: &str,
) {
    tracing::error!(
        series_id,
        season_number,
        ?episode_number,
        error,
        "series target search failed"
    );
    let ep = episode_number.unwrap_or(0);
    let _ = sqlx::query(
        r#"
        INSERT INTO series_target_state(
          series_id,season_number,episode_number,last_search_at,last_error,status
        ) VALUES(?,?,?,CURRENT_TIMESTAMP,?,'error')
        ON CONFLICT(series_id,season_number,episode_number) DO UPDATE SET
          last_search_at=CURRENT_TIMESTAMP,last_error=excluded.last_error,status='error'
    "#,
    )
    .bind(series_id)
    .bind(season_number)
    .bind(ep)
    .bind(error)
    .execute(&state.db)
    .await;
}

pub(super) async fn touch_search(state: &AppState, media_type: &str, media_id: i64, status: &str) {
    let _ = sqlx::query(
        r#"
        INSERT INTO automation_state(media_type,media_id,last_search_at,status)
        VALUES(?,?,CURRENT_TIMESTAMP,?)
        ON CONFLICT(media_type,media_id) DO UPDATE SET
          last_search_at=CURRENT_TIMESTAMP,status=excluded.status,last_error=NULL
    "#,
    )
    .bind(media_type)
    .bind(media_id)
    .bind(status)
    .execute(&state.db)
    .await;
}

pub(super) async fn record_error(state: &AppState, media_type: &str, media_id: i64, error: &str) {
    tracing::error!(media_type, media_id, error, "automation search failed");
    let _ = sqlx::query(
        r#"
        INSERT INTO automation_state(media_type,media_id,last_search_at,last_error,status)
        VALUES(?,?,CURRENT_TIMESTAMP,?,'error')
        ON CONFLICT(media_type,media_id) DO UPDATE SET
          last_search_at=CURRENT_TIMESTAMP,last_error=excluded.last_error,status='error'
    "#,
    )
    .bind(media_type)
    .bind(media_id)
    .bind(error)
    .execute(&state.db)
    .await;
}
