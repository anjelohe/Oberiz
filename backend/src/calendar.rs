use crate::AppState;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CalendarQuery {
    pub days: Option<i64>,
    pub include_past: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CalendarEpisode {
    pub series_id: i64,
    pub series_name: String,
    pub poster_path: Option<String>,
    pub season_number: i32,
    pub episode_number: i32,
    pub episode_name: String,
    pub air_date: String,
    pub monitored: bool,
    pub has_file: bool,
}

pub async fn list_calendar(
    State(state): State<AppState>,
    Query(query): Query<CalendarQuery>,
) -> Result<Json<Vec<CalendarEpisode>>, (StatusCode, String)> {
    let days = query.days.unwrap_or(45).clamp(1, 365);
    let back = if query.include_past.unwrap_or(true) {
        7
    } else {
        0
    };
    let rows=sqlx::query_as::<_,CalendarEpisode>(r#"
        SELECT s.id AS series_id,s.name AS series_name,s.poster_path,
               e.season_number,e.episode_number,e.name AS episode_name,e.air_date,e.monitored,e.has_file
        FROM series_episodes e JOIN series s ON s.id=e.series_id
        WHERE e.air_date IS NOT NULL
          AND e.air_date>=date('now','localtime', ?)
          AND e.air_date<=date('now','localtime', ?)
        ORDER BY e.air_date,s.name,e.season_number,e.episode_number
    "#)
        .bind(format!("-{} days",back)).bind(format!("+{} days",days))
        .fetch_all(&state.db).await.map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()))?;
    Ok(Json(rows))
}
