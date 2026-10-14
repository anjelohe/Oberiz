use axum::{Router, routing::get, routing::put};

use crate::{AppState, profiles};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/profiles",
            get(profiles::list_quality_profiles).post(profiles::create_quality_profile),
        )
        .route(
            "/api/profiles/{id}",
            put(profiles::update_quality_profile).delete(profiles::delete_quality_profile),
        )
        .route(
            "/api/profiles/{id}/default",
            put(profiles::set_default_quality_profile),
        )
        .route(
            "/api/language-profiles",
            get(profiles::list_language_profiles).post(profiles::create_language_profile),
        )
        .route(
            "/api/language-profiles/{id}",
            put(profiles::update_language_profile).delete(profiles::delete_language_profile),
        )
}
