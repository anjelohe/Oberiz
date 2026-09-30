use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{AppState, series, tmdb};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/series/search", get(tmdb::search_series))
        .route(
            "/api/series",
            get(series::list_series).post(series::create_series),
        )
        .route(
            "/api/series/{id}",
            get(series::get_series)
                .put(series::update_series)
                .delete(series::delete_series),
        )
        .route("/api/series/{id}/refresh", post(series::refresh_series))
        .route("/api/series/{id}/seasons", get(series::list_seasons))
        .route(
            "/api/series/{id}/seasons/{season_number}",
            put(series::update_season),
        )
        .route(
            "/api/series/{id}/episodes/{episode_id}",
            put(series::update_episode),
        )
}
