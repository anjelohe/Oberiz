use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct HistoryItem {
    pub id: i64,
    pub event_type: String,
    pub title: String,
    pub detail: Option<String>,
    pub level: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
}

pub async fn record(
    db: &sqlx::SqlitePool,
    event_type: &str,
    title: &str,
    detail: Option<&str>,
    level: &str,
) {
    let _ =
        sqlx::query("INSERT INTO history (event_type, title, detail, level) VALUES (?, ?, ?, ?)")
            .bind(event_type)
            .bind(title)
            .bind(detail)
            .bind(level)
            .execute(db)
            .await;
}

pub async fn list_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<Vec<HistoryItem>>, (StatusCode, String)> {
    let limit = query.limit.unwrap_or(100).clamp(1, 500);

    let rows = sqlx::query_as::<_, HistoryItem>(
        r#"
        SELECT id, event_type, title, detail, level, created_at
        FROM history
        ORDER BY id DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows))
}
