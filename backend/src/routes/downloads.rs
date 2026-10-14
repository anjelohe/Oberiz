use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::{AppState, qbittorrent, rejections, releases, search_api};

pub fn router() -> Router<AppState> {
    Router::new()
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
        .route("/api/releases/search", get(search_api::search))
        .route("/api/releases/grab", post(search_api::grab))
        .route("/api/releases/parse", get(releases::parse_release))
        .route("/api/releases/reject", post(rejections::reject_release))
        .route("/api/releases/rejected", get(rejections::list_rejected))
        .route(
            "/api/releases/rejected/{id}",
            delete(rejections::unreject_release),
        )
        .route(
            "/api/downloads/{hash}/reject",
            post(rejections::reject_torrent),
        )
}
