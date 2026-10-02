//! Releases the user has rejected by hand for a given movie or series.
//!
//! A rejection is keyed on `(media_type, media_id, release_key)`, where the key
//! is the release title normalized by [`release_key`]. Search results have no
//! infohash until the torrent is fetched, so the title is the identity shared
//! by a search result, an RSS item and a past download job.

use crate::{AppState, history};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reason attached to a search result that matched a manual rejection.
pub const MANUAL_REJECTION_REASON: &str = "Rechazado manualmente";

/// Lowercased title with every run of non-alphanumeric characters collapsed
/// into a single space, so punctuation, dots-vs-spaces and case differences
/// between indexers don't make the same release look different.
pub fn release_key(title: &str) -> String {
    let mut key = String::with_capacity(title.len());
    let mut pending_space = false;
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            if pending_space && !key.is_empty() {
                key.push(' ');
            }
            pending_space = false;
            key.extend(ch.to_lowercase());
        } else {
            pending_space = true;
        }
    }
    key
}

pub async fn rejected_keys(
    db: &sqlx::SqlitePool,
    media_type: &str,
    media_id: i64,
) -> HashMap<String, i64> {
    sqlx::query_as::<_, (String, i64)>(
        "SELECT release_key,id FROM rejected_releases WHERE media_type=? AND media_id=?",
    )
    .bind(media_type)
    .bind(media_id)
    .fetch_all(db)
    .await
    .map(|rows| rows.into_iter().collect())
    .unwrap_or_default()
}

pub async fn is_rejected(
    db: &sqlx::SqlitePool,
    media_type: &str,
    media_id: i64,
    title: &str,
) -> bool {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM rejected_releases WHERE media_type=? AND media_id=? AND release_key=?",
    )
    .bind(media_type)
    .bind(media_id)
    .bind(release_key(title))
    .fetch_one(db)
    .await
    .unwrap_or(0)
        > 0
}

/// Drops every rejection of a movie/series that is being deleted, so a later
/// title that reuses its numeric id doesn't inherit them.
pub async fn forget_media(db: &sqlx::SqlitePool, media_type: &str, media_id: i64) {
    let _ = sqlx::query("DELETE FROM rejected_releases WHERE media_type=? AND media_id=?")
        .bind(media_type)
        .bind(media_id)
        .execute(db)
        .await;
}

async fn insert_rejection(
    db: &sqlx::SqlitePool,
    media_type: &str,
    media_id: i64,
    title: &str,
    indexer_name: Option<&str>,
) -> Result<i64, (StatusCode, String)> {
    let key = release_key(title);
    if key.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "El título del release está vacío".into(),
        ));
    }
    // Re-rejecting an already rejected release is a no-op that still yields
    // its id (the UPDATE is only there so RETURNING fires on conflict).
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO rejected_releases(media_type,media_id,release_key,release_title,indexer_name) \
         VALUES(?,?,?,?,?) \
         ON CONFLICT(media_type,media_id,release_key) DO UPDATE SET release_key=release_key \
         RETURNING id",
    )
    .bind(media_type)
    .bind(media_id)
    .bind(key)
    .bind(title)
    .bind(indexer_name)
    .fetch_one(db)
    .await
    .map_err(internal)
}

fn validate_target(media_type: &str, media_id: i64) -> Result<(), (StatusCode, String)> {
    if media_type != "movie" && media_type != "series" {
        return Err((
            StatusCode::BAD_REQUEST,
            "media_type debe ser movie o series".into(),
        ));
    }
    if media_id <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Hace falta una película o serie de destino para rechazar un release".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct RejectRequest {
    pub media_type: String,
    pub media_id: i64,
    pub title: String,
    pub indexer_name: Option<String>,
}

pub async fn reject_release(
    State(state): State<AppState>,
    Json(req): Json<RejectRequest>,
) -> Result<Json<RejectResponse>, (StatusCode, String)> {
    validate_target(&req.media_type, req.media_id)?;
    let id = insert_rejection(
        &state.db,
        &req.media_type,
        req.media_id,
        &req.title,
        req.indexer_name.as_deref(),
    )
    .await?;
    history::record(
        &state.db,
        "release.rejected",
        &req.title,
        Some("Rechazado manualmente; no se volverá a descargar para este título."),
        "info",
    )
    .await;
    Ok(Json(RejectResponse { id }))
}

#[derive(Debug, Serialize)]
pub struct RejectResponse {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct RejectedQuery {
    pub media_type: Option<String>,
    pub media_id: Option<i64>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RejectedRelease {
    pub id: i64,
    pub media_type: String,
    pub media_id: i64,
    pub media_title: Option<String>,
    pub release_title: String,
    pub indexer_name: Option<String>,
    pub created_at: String,
}

pub async fn list_rejected(
    State(state): State<AppState>,
    Query(query): Query<RejectedQuery>,
) -> Result<Json<Vec<RejectedRelease>>, (StatusCode, String)> {
    list_rows(&state.db, query.media_type, query.media_id)
        .await
        .map(Json)
        .map_err(internal)
}

async fn list_rows(
    db: &sqlx::SqlitePool,
    media_type: Option<String>,
    media_id: Option<i64>,
) -> Result<Vec<RejectedRelease>, sqlx::Error> {
    sqlx::query_as::<_, RejectedRelease>(
        r#"
        SELECT r.id, r.media_type, r.media_id,
               CASE r.media_type
                 WHEN 'movie' THEN (SELECT title FROM movies WHERE id=r.media_id)
                 ELSE (SELECT name FROM series WHERE id=r.media_id)
               END AS media_title,
               r.release_title, r.indexer_name, r.created_at
        FROM rejected_releases r
        WHERE (?1 IS NULL OR r.media_type=?1) AND (?2 IS NULL OR r.media_id=?2)
        ORDER BY r.id DESC
        LIMIT 500
        "#,
    )
    .bind(media_type)
    .bind(media_id)
    .fetch_all(db)
    .await
}

pub async fn unreject_release(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let removed = sqlx::query_as::<_, (String,)>(
        "DELETE FROM rejected_releases WHERE id=? RETURNING release_title",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?;
    let Some((title,)) = removed else {
        return Err((StatusCode::NOT_FOUND, "Rechazo no encontrado".into()));
    };
    history::record(
        &state.db,
        "release.unrejected",
        &title,
        Some("Rechazo manual retirado; el release vuelve a poder elegirse."),
        "info",
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct RejectTorrentRequest {
    #[serde(default)]
    pub delete_files: bool,
}

/// Rejects the release behind an Oberiz-managed torrent and removes it from
/// qBittorrent. Torrents without an Oberiz job are refused: Oberiz never chose
/// them, so it has no title to reject.
pub async fn reject_torrent(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Json(req): Json<RejectTorrentRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let job = sqlx::query_as::<_, (i64, String, i64, String, Option<String>)>(
        "SELECT id,media_type,media_id,release_title,indexer_name FROM download_jobs \
         WHERE lower(qb_hash)=lower(?) AND media_id>0 ORDER BY id DESC LIMIT 1",
    )
    .bind(&hash)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?;
    let Some((job_id, media_type, media_id, title, indexer_name)) = job else {
        return Err((
            StatusCode::NOT_FOUND,
            "Este torrent no lo descargó Oberiz, así que no hay nada que rechazar".into(),
        ));
    };

    insert_rejection(
        &state.db,
        &media_type,
        media_id,
        &title,
        indexer_name.as_deref(),
    )
    .await?;
    state
        .download_client
        .delete_torrent(&state, &hash, req.delete_files)
        .await?;
    crate::qbittorrent::invalidate_torrent_cache(&state).await;
    sqlx::query(
        "UPDATE download_jobs SET status='rejected',last_error=NULL,updated_at=CURRENT_TIMESTAMP WHERE id=?",
    )
    .bind(job_id)
    .execute(&state.db)
    .await
    .map_err(internal)?;
    history::record(
        &state.db,
        "release.rejected",
        &title,
        Some(if req.delete_files {
            "Rechazado manualmente; torrent y archivos eliminados de qBittorrent."
        } else {
            "Rechazado manualmente; torrent eliminado de qBittorrent (archivos conservados)."
        }),
        "warning",
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[cfg(test)]
mod tests {
    use super::release_key;

    #[test]
    fn key_ignores_case_and_punctuation() {
        assert_eq!(
            release_key("Resident.Evil_Welcome-to  Raccoon City (2021) 2160p"),
            release_key("resident evil welcome to raccoon city 2021 2160p")
        );
    }

    #[test]
    fn key_keeps_distinct_releases_apart() {
        assert_ne!(
            release_key("Show S01E02 1080p WEB-DL"),
            release_key("Show S01E02 2160p WEB-DL")
        );
    }

    #[test]
    fn key_of_punctuation_only_is_empty() {
        assert_eq!(release_key(" -- .. "), "");
    }

    async fn pool() -> sqlx::SqlitePool {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(include_str!("../migrations/0020_rejected_releases.sql"))
            .execute(&db)
            .await
            .unwrap();
        db
    }

    #[tokio::test]
    async fn rejection_is_scoped_to_one_media_and_release() {
        let db = pool().await;
        let first =
            super::insert_rejection(&db, "movie", 7, "Resident.Evil.2021.2160p", Some("Idx"))
                .await
                .unwrap();
        // Re-rejecting the same release (different punctuation) is a no-op
        // that returns the same id.
        let again = super::insert_rejection(&db, "movie", 7, "resident evil 2021 2160p", None)
            .await
            .unwrap();
        assert_eq!(first, again);

        assert!(super::is_rejected(&db, "movie", 7, "Resident Evil 2021 2160p").await);
        assert!(!super::is_rejected(&db, "movie", 8, "Resident Evil 2021 2160p").await);
        assert!(!super::is_rejected(&db, "series", 7, "Resident Evil 2021 2160p").await);
        assert!(!super::is_rejected(&db, "movie", 7, "Resident Evil 2026 2160p").await);
        assert_eq!(super::rejected_keys(&db, "movie", 7).await.len(), 1);

        super::forget_media(&db, "movie", 7).await;
        assert!(!super::is_rejected(&db, "movie", 7, "Resident Evil 2021 2160p").await);
    }

    #[tokio::test]
    async fn listing_filters_and_names_the_media() {
        let db = pool().await;
        sqlx::query("CREATE TABLE movies(id INTEGER PRIMARY KEY, title TEXT)")
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE series(id INTEGER PRIMARY KEY, name TEXT)")
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO movies VALUES(7,'Resident Evil')")
            .execute(&db)
            .await
            .unwrap();
        super::insert_rejection(&db, "movie", 7, "Resident Evil 2021", Some("Idx"))
            .await
            .unwrap();
        super::insert_rejection(&db, "series", 3, "Gone Show S01", None)
            .await
            .unwrap();

        let all = super::list_rows(&db, None, None).await.unwrap();
        assert_eq!(all.len(), 2);
        let movie = super::list_rows(&db, Some("movie".into()), Some(7))
            .await
            .unwrap();
        assert_eq!(movie.len(), 1);
        assert_eq!(movie[0].media_title.as_deref(), Some("Resident Evil"));
        // A rejection whose series row no longer exists still lists, unnamed.
        let series = super::list_rows(&db, Some("series".into()), None)
            .await
            .unwrap();
        assert_eq!(series.len(), 1);
        assert_eq!(series[0].media_title, None);
    }

    #[tokio::test]
    async fn empty_title_cannot_be_rejected() {
        let db = pool().await;
        assert!(
            super::insert_rejection(&db, "movie", 1, " -- ", None)
                .await
                .is_err()
        );
    }
}
