use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, put},
};
use tower_http::cors::{Any, CorsLayer};

use crate::{AppState, external_library, overseerr, public_api};

async fn authorize_before_body(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::response::IntoResponse;
    if request.method() == axum::http::Method::OPTIONS {
        return next.run(request).await;
    }
    let authorization = if request.uri().path().starts_with("/api/v1/") {
        public_api::authorize(&state, request.headers()).await
    } else {
        overseerr::authorize(&state, request.headers(), request.uri()).await
    };
    if let Err(error) = authorization {
        return error.into_response();
    }
    if request.uri().path().starts_with("/api/v1/library/missing")
        && let Err(error) = crate::missing::authorize_external(&state).await
    {
        return error.into_response();
    }
    static IN_FLIGHT: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(4);
    let Ok(_permit) = IN_FLIGHT.try_acquire() else {
        return (
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "Too many concurrent integration requests",
        )
            .into_response();
    };
    next.run(request).await
}

/// Integration routes authenticate before any body extractor runs.
pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/v1/status", get(public_api::status))
        .route("/api/v1/library/missing", get(crate::missing::read))
        .route(
            "/api/v1/library/missing/refresh",
            axum::routing::post(crate::missing::refresh).layer(DefaultBodyLimit::max(4096)),
        )
        .route(
            "/api/v1/quality-profiles",
            get(public_api::list_quality_profiles),
        )
        .route(
            "/api/v1/requests",
            get(public_api::list_requests).post(public_api::create_request),
        )
        .route("/api/v1/requests/{id}", get(public_api::get_request))
        // A full library snapshot: tens of thousands of episodes fit well
        // within this, while still bounding what one request can allocate.
        .route(
            "/api/v1/library",
            put(external_library::report).layer(DefaultBodyLimit::max(128 * 1024 * 1024)),
        )
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
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            authorize_before_body,
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn invalid_json_is_not_read_before_api_authorization() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        let state = crate::AppState {
            db: db.clone(),
            http: reqwest::Client::new(),
            automation_runtime: Default::default(),
            download_client: std::sync::Arc::new(crate::download_client::QBittorrentClient),
            torrent_list_cache: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            grab_lock: std::sync::Arc::new(tokio::sync::Mutex::new(())),
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/api/v1/library", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, super::router(state.clone()).with_state(state))
                .await
                .unwrap()
        });
        let client = reqwest::Client::new();
        let response = client
            .put(&url)
            .header("content-type", "application/json")
            .body("not json")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
        crate::settings::set_value(&db, "api.enabled", "true")
            .await
            .unwrap();
        crate::settings::set_value(
            &db,
            "api.key_hash",
            &crate::settings::hash_api_key("test-only-key"),
        )
        .await
        .unwrap();
        let response = client
            .put(&url)
            .header("content-type", "application/json")
            .header("x-api-key", "wrong-key")
            .body("not json")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
        let response = client
            .put(&url)
            .header("content-type", "application/json")
            .header("x-api-key", "test-only-key")
            .body("not json")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
        task.abort();
    }
}
