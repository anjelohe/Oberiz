mod automation;
mod backups;
mod calendar;
mod cardigann;
mod db;
mod diagnostics;
mod filesystem;
mod history;
mod importer;
mod indexers;
mod library;
mod movies;
mod overseerr;
mod profiles;
mod public_api;
mod qbittorrent;
mod releases;
mod rss;
mod search_api;
mod series;
mod settings;
mod tmdb;

use automation::AutomationRuntime;
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
};
use serde::Serialize;
use sqlx::SqlitePool;
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    http: reqwest::Client,
    automation_runtime: AutomationRuntime,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    name: &'static str,
    version: &'static str,
    database: &'static str,
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let database = if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok()
    {
        "ok"
    } else {
        "error"
    };

    Json(HealthResponse {
        status: "ok",
        name: "Oberiz",
        version: env!("CARGO_PKG_VERSION"),
        database,
    })
}

async fn frontend_shell() -> Response {
    let static_directory =
        std::env::var("OBERIZ_STATIC_DIR").unwrap_or_else(|_| "frontend".to_string());
    match tokio::fs::read_to_string(std::path::Path::new(&static_directory).join("index.html"))
        .await
    {
        Ok(contents) => Html(contents).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Oberiz web interface is not installed.",
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db = db::connect().await?;
    let http = reqwest::Client::builder()
        .user_agent("Oberiz/1.0.0")
        .build()?;
    let state = AppState {
        db,
        http,
        automation_runtime: AutomationRuntime::default(),
    };
    automation::spawn_scheduler(state.clone());
    backups::spawn_scheduler(state.clone());
    rss::spawn_scheduler(state.clone());
    importer::spawn_import_scheduler(state.clone());
    indexers::spawn_health_scheduler(state.clone());

    let static_directory = std::path::PathBuf::from(
        std::env::var("OBERIZ_STATIC_DIR").unwrap_or_else(|_| "frontend".to_string()),
    );
    let assets_directory = static_directory.join("assets");

    let app = Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/filesystem/directories",
            get(filesystem::list_directories),
        )
        .route(
            "/api/settings",
            get(settings::get_settings).put(settings::update_settings),
        )
        .route(
            "/api/settings/test-qbittorrent",
            post(qbittorrent::test_connection),
        )
        .route(
            "/api/backups",
            get(backups::list_backups).post(backups::create_backup),
        )
        .route(
            "/api/backups/upload",
            post(backups::upload_backup).layer(DefaultBodyLimit::max(1024 * 1024 * 1024)),
        )
        .route(
            "/api/backups/{filename}/download",
            get(backups::download_backup),
        )
        .route("/api/backups/{filename}", delete(backups::delete_backup))
        .route(
            "/api/backups/{filename}/restore",
            post(backups::restore_backup),
        )
        .route("/api/diagnostics", get(diagnostics::get_diagnostics))
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
        .route("/api/downloads", get(qbittorrent::list_downloads))
        .route(
            "/api/qbittorrent/categories",
            get(qbittorrent::list_categories).post(qbittorrent::create_category),
        )
        .route(
            "/api/qbittorrent/categories/{name}",
            put(qbittorrent::update_category).delete(qbittorrent::delete_category),
        )
        .route(
            "/api/qbittorrent/tags",
            get(qbittorrent::list_tags)
                .post(qbittorrent::create_tags)
                .delete(qbittorrent::delete_tags),
        )
        .route("/api/downloads/magnet", post(qbittorrent::add_magnet))
        .route(
            "/api/downloads/{hash}/organization",
            put(qbittorrent::update_torrent_organization),
        )
        .route(
            "/api/downloads/{hash}/stop",
            post(qbittorrent::stop_torrent),
        )
        .route(
            "/api/downloads/{hash}/start",
            post(qbittorrent::start_torrent),
        )
        .route("/api/downloads/{hash}", delete(qbittorrent::delete_torrent))
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
        .route("/api/releases/search", get(search_api::search))
        .route("/api/releases/grab", post(search_api::grab))
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
        .route("/api/releases/parse", get(releases::parse_release))
        .route("/api/history", get(history::list_history))
        .route("/api/library/rescan", post(library::rescan))
        .route("/api/library/scans", get(library::scan_history))
        .route("/api/library/summary", get(library::summary))
        .route("/api/calendar", get(calendar::list_calendar))
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
        .nest_service("/assets", ServeDir::new(assets_directory))
        .route_service(
            "/oberiz-logo.png",
            ServeFile::new(static_directory.join("oberiz-logo.png")),
        )
        .route_service(
            "/oberiz-logo-middle.png",
            ServeFile::new(static_directory.join("oberiz-logo-middle.png")),
        )
        .route_service(
            "/oberiz-logo-premium.png",
            ServeFile::new(static_directory.join("oberiz-logo-premium.png")),
        )
        .with_state(state)
        .fallback(get(frontend_shell));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2032").await?;
    println!(
        "Oberiz v{} running on http://localhost:2032",
        env!("CARGO_PKG_VERSION")
    );
    println!(
        "Movies, Series, Profiles, Automation, Importer/Reseed, Downloads, Indexers and History enabled"
    );

    axum::serve(listener, app).await?;
    Ok(())
}
