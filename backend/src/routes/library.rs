use axum::{
    Router,
    routing::{get, post},
};

use crate::{AppState, calendar, history, library};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/library/missing", get(crate::missing::read))
        .route(
            "/api/library/missing/collection",
            get(crate::missing::movie_collection),
        )
        .route(
            "/api/library/missing/refresh",
            post(crate::missing::refresh),
        )
        .route("/api/history", get(history::list_history))
        .route("/api/library/rescan", post(library::rescan))
        .route("/api/library/scans", get(library::scan_history))
        .route("/api/library/roots", get(library::root_status))
        .route("/api/library/summary", get(library::summary))
        .route("/api/calendar", get(calendar::list_calendar))
}
