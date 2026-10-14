use axum::{Router, routing::get};
use tower_http::cors::{Any, CorsLayer};

use crate::{AppState, overseerr, public_api};

/// Kept on its own router with a permissive CORS policy: these routes are
/// meant to be called by external clients (Cinetta, Overseerr) from any
/// origin, and are already gated by the `api.key` header check in
/// `public_api` / `overseerr`, independently of the admin-password
/// protection applied to the internal router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/status", get(public_api::status))
        .route(
            "/api/v1/quality-profiles",
            get(public_api::list_quality_profiles),
        )
        .route(
            "/api/v1/requests",
            get(public_api::list_requests).post(public_api::create_request),
        )
        .route("/api/v1/requests/{id}", get(public_api::get_request))
        .route(
            "/radarr/api/v3/system/status",
            get(overseerr::radarr_status),
        )
        .route(
            "/radarr/api/v3/qualityprofile",
            get(overseerr::radarr_profiles),
        )
        .route(
            "/radarr/api/v3/qualityProfile",
            get(overseerr::radarr_profiles),
        )
        .route("/radarr/api/v3/rootfolder", get(overseerr::radarr_roots))
        .route("/radarr/api/v3/rootFolder", get(overseerr::radarr_roots))
        .route("/radarr/api/v3/tag", get(overseerr::tags))
        .route(
            "/radarr/api/v3/languageprofile",
            get(overseerr::language_profiles),
        )
        .route("/radarr/api/v3/movie/lookup", get(overseerr::radarr_lookup))
        .route(
            "/radarr/api/v3/movie",
            get(overseerr::radarr_movies).post(overseerr::radarr_add_movie),
        )
        .route(
            "/sonarr/api/v3/system/status",
            get(overseerr::sonarr_status),
        )
        .route(
            "/sonarr/api/v3/qualityprofile",
            get(overseerr::sonarr_profiles),
        )
        .route(
            "/sonarr/api/v3/qualityProfile",
            get(overseerr::sonarr_profiles),
        )
        .route("/sonarr/api/v3/rootfolder", get(overseerr::sonarr_roots))
        .route("/sonarr/api/v3/rootFolder", get(overseerr::sonarr_roots))
        .route("/sonarr/api/v3/tag", get(overseerr::tags))
        .route(
            "/sonarr/api/v3/languageprofile",
            get(overseerr::language_profiles),
        )
        .route(
            "/sonarr/api/v3/series/lookup",
            get(overseerr::sonarr_lookup),
        )
        .route(
            "/sonarr/api/v3/series",
            get(overseerr::sonarr_series).post(overseerr::sonarr_add_series),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}
