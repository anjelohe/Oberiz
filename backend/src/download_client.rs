//! The download-client abstraction Oberiz's own domain logic (automation,
//! importer, search/grab) talks to, instead of calling `qbittorrent::*`
//! directly. qBittorrent is the only implementation today, but nothing
//! outside this module and `qbittorrent.rs` itself needs to know that: a
//! second client (Transmission, Deluge, ...) only has to implement this
//! trait and map its own torrent shape into `QBittorrentTorrent`, without
//! touching automation/importer/search_api at all.
use crate::{AppState, qbittorrent::QBittorrentTorrent};
use async_trait::async_trait;
use axum::http::StatusCode;

#[async_trait]
pub trait DownloadClient: Send + Sync {
    async fn list_torrents(
        &self,
        state: &AppState,
    ) -> Result<Vec<QBittorrentTorrent>, (StatusCode, String)>;

    async fn add_magnet(
        &self,
        state: &AppState,
        url: &str,
        category: &str,
        title: &str,
        user_tags: &str,
        job_id: Option<i64>,
    ) -> Result<String, (StatusCode, String)>;

    #[allow(clippy::too_many_arguments)]
    async fn add_torrent_file(
        &self,
        state: &AppState,
        bytes: Vec<u8>,
        category: &str,
        title: &str,
        user_tags: &str,
        job_id: Option<i64>,
        save_path: Option<&str>,
        reseed: bool,
    ) -> Result<String, (StatusCode, String)>;

    async fn export_torrent_file(
        &self,
        state: &AppState,
        hash: &str,
    ) -> Result<Vec<u8>, (StatusCode, String)>;

    async fn delete_torrent(
        &self,
        state: &AppState,
        hash: &str,
        delete_files: bool,
    ) -> Result<(), (StatusCode, String)>;
}

pub struct QBittorrentClient;

#[async_trait]
impl DownloadClient for QBittorrentClient {
    async fn list_torrents(
        &self,
        state: &AppState,
    ) -> Result<Vec<QBittorrentTorrent>, (StatusCode, String)> {
        crate::qbittorrent::list_torrents_internal(state).await
    }

    async fn add_magnet(
        &self,
        state: &AppState,
        url: &str,
        category: &str,
        title: &str,
        user_tags: &str,
        job_id: Option<i64>,
    ) -> Result<String, (StatusCode, String)> {
        crate::qbittorrent::add_url(state, url, title, category, user_tags, job_id).await
    }

    async fn add_torrent_file(
        &self,
        state: &AppState,
        bytes: Vec<u8>,
        category: &str,
        title: &str,
        user_tags: &str,
        job_id: Option<i64>,
        save_path: Option<&str>,
        reseed: bool,
    ) -> Result<String, (StatusCode, String)> {
        crate::qbittorrent::add_torrent_bytes_with_options(
            state, bytes, category, title, user_tags, job_id, save_path, reseed,
        )
        .await
    }

    async fn export_torrent_file(
        &self,
        state: &AppState,
        hash: &str,
    ) -> Result<Vec<u8>, (StatusCode, String)> {
        crate::qbittorrent::export_torrent(state, hash).await
    }

    async fn delete_torrent(
        &self,
        state: &AppState,
        hash: &str,
        delete_files: bool,
    ) -> Result<(), (StatusCode, String)> {
        crate::qbittorrent::delete_torrent_internal(state, hash, delete_files).await
    }
}
