use axum::{Router, routing::get};

use crate::{AppState, movies, tmdb};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/movies/search", get(tmdb::search_movies))
        .route(
            "/api/movies",
            get(movies::list_movies).post(movies::create_movie),
        )
        .route(
            "/api/movies/{id}",
            get(movies::get_movie)
                .put(movies::update_movie)
                .delete(movies::delete_movie),
        )
}
