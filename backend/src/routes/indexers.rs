use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{AppState, indexers};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/indexers", get(indexers::list_indexers))
        .route("/api/indexers/sync", post(indexers::sync_upstream))
        .route("/api/indexers/{id}/enabled", put(indexers::set_enabled))
        .route(
            "/api/indexers/{id}",
            get(indexers::get_indexer).delete(indexers::remove_indexer),
        )
        .route(
            "/api/indexers/{id}/config",
            put(indexers::save_indexer_config),
        )
        .route("/api/indexers/{id}/test", post(indexers::test_indexer))
}
