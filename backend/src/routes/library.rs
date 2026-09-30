use axum::{
    Router,
    routing::{get, post},
};

use crate::{AppState, calendar, history, library};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/history", get(history::list_history))
        .route("/api/library/rescan", post(library::rescan))
        .route("/api/library/scans", get(library::scan_history))
        .route("/api/library/summary", get(library::summary))
        .route("/api/calendar", get(calendar::list_calendar))
}
