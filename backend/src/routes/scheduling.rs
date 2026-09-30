use axum::{
    Router,
    routing::{get, post, put},
};

use crate::{AppState, automation, importer, rss};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/automation/status", get(automation::status))
        .route("/api/automation/run", post(automation::run_now))
        .route("/api/rss/status", get(rss::status))
        .route("/api/rss/run", post(rss::run_now))
        .route(
            "/api/imports",
            get(importer::list_imports).delete(importer::delete_imports),
        )
        .route("/api/imports/{id}/reseed", post(importer::reseed))
        .route("/api/seed-policies", get(importer::list_seed_policies))
        .route(
            "/api/seed-policies/{indexer_id}",
            put(importer::upsert_seed_policy),
        )
}
