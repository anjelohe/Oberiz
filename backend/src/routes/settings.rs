use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
};

use crate::{AppState, backups, diagnostics, filesystem, qbittorrent, settings};

pub fn router() -> Router<AppState> {
    Router::new()
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
}
