use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize)]
pub struct Diagnostics {
    pub version: &'static str,
    pub operating_system: &'static str,
    pub database: &'static str,
    pub tmdb_configured: bool,
    pub qbittorrent_configured: bool,
    pub enabled_indexers: i64,
    pub total_indexers: i64,
    pub automation_enabled: bool,
    pub recent_errors: Vec<DiagnosticError>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct DiagnosticError {
    pub source: String,
    pub message: String,
    pub occurred_at: String,
}

pub async fn get_diagnostics(
    State(state): State<AppState>,
) -> Result<Json<Diagnostics>, (StatusCode, String)> {
    let database = if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok()
    {
        "ok"
    } else {
        "error"
    };
    let tmdb_configured = !setting(&state.db, "tmdb.api_key").await.is_empty();
    let qbittorrent_configured = !setting(&state.db, "qbittorrent.host").await.is_empty();
    let automation_enabled = setting(&state.db, "automation.enabled").await == "true";
    let enabled_indexers =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM indexer_configs WHERE enabled=1")
            .fetch_one(&state.db)
            .await
            .map_err(internal)?;
    let total_indexers = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM indexer_configs")
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;
    let recent_errors = sqlx::query_as::<_, DiagnosticError>(
        r#"
        SELECT event_type AS source, COALESCE(detail,title) AS message, created_at AS occurred_at
        FROM history WHERE level='error' ORDER BY created_at DESC LIMIT 20
    "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;
    Ok(Json(Diagnostics {
        version: env!("CARGO_PKG_VERSION"),
        operating_system: std::env::consts::OS,
        database,
        tmdb_configured,
        qbittorrent_configured,
        enabled_indexers,
        total_indexers,
        automation_enabled,
        recent_errors,
    }))
}

fn internal<E: std::fmt::Display>(error: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

async fn setting(db: &sqlx::SqlitePool, key: &str) -> String {
    crate::settings::get_value(db, key)
        .await
        .ok()
        .flatten()
        .unwrap_or_default()
}
