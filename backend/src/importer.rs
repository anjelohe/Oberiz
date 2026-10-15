//! Download tracking and sharing policies; no direct media-file operations.
mod naming;
mod remote;
#[cfg(test)]
mod workflow_tests;
use crate::{AppState, history, qbittorrent, settings};
use axum::{
    Json,
    extract::{Path as AxumPath, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use std::collections::HashSet;
use tokio::time::{Duration, sleep};
pub(crate) static MEDIA_OPERATION_GATE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
pub(crate) async fn external_mode(state: &AppState) -> Result<bool, (StatusCode, String)> {
    if let Some(source) = settings::get_value(&state.db, "library.source")
        .await
        .map_err(internal)?
    {
        return Ok(source != "local");
    }
    Ok(settings::get_value(&state.db, "import.method")
        .await
        .map_err(internal)?
        .is_some_and(|method| method.trim().eq_ignore_ascii_case("external")))
}
pub(crate) async fn require_managed_files(state: &AppState) -> Result<(), (StatusCode, String)> {
    if external_mode(state).await? {
        return Err((
            StatusCode::FORBIDDEN,
            "Local scanning disabled: library source is Cinetta".into(),
        ));
    }
    Ok(())
}
pub(crate) async fn translate_remote_path(_: &AppState, path: &str) -> String {
    path.to_owned()
}

/// Marks jobs missing only after qBittorrent answered successfully.
/// This also clears an old queued job that never received a qBittorrent hash, so a failed
/// hand-off cannot indefinitely stop automation from trying the media again.
pub(crate) async fn reconcile_missing_torrents(
    state: &AppState,
) -> Result<usize, (StatusCode, String)> {
    let torrents = state.download_client.list_torrents(state).await?;
    let present: HashSet<String> = torrents.into_iter().map(|torrent| torrent.hash).collect();
    // A 'queued' job gets the two-minute grace whether or not it already has
    // a hash: grab_internal now records the infohash on the job *before*
    // handing the torrent to qBittorrent, so a queued job with a hash can
    // simply be one qBittorrent hasn't finished adding yet.
    let jobs = sqlx::query_as::<_, (i64, String, Option<String>)>(
        r#"
        SELECT id,release_title,qb_hash FROM download_jobs
        WHERE status IN ('queued','downloading','completed','seeding','restored_stopped')
          AND (
            (status<>'queued' AND qb_hash IS NOT NULL AND qb_hash<>'')
            OR (
              status='queued'
              AND updated_at < datetime('now', '-2 minutes')
            )
          )
    "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;
    let mut missing = 0;
    for (id, title, hash) in jobs {
        if hash.as_ref().is_some_and(|value| present.contains(value)) {
            continue;
        }
        let reason = if hash.is_some() {
            "Torrent no longer exists in qBittorrent"
        } else {
            "Queued job never received a qBittorrent hash"
        };
        sqlx::query("UPDATE download_jobs SET status='missing',last_error=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(reason)
            .bind(id).execute(&state.db).await.map_err(internal)?;
        history::record(
            &state.db,
            "download.missing",
            &title,
            Some("No active torrent was found in qBittorrent; it can be searched again."),
            "warning",
        )
        .await;
        missing += 1;
    }
    Ok(missing)
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DownloadJob {
    pub id: i64,
    pub media_type: String,
    pub media_id: i64,
    pub release_title: String,
    pub indexer_id: String,
    pub indexer_name: Option<String>,
    pub category: String,
    pub qbittorrent_tags: String,
    pub season_number: Option<i32>,
    pub episode_number: Option<i32>,
    pub is_season_pack: bool,
    pub qb_hash: Option<String>,
    pub status: String,
    pub source_path: Option<String>,
    pub library_path: Option<String>,
    pub import_method: Option<String>,
    pub file_mappings_json: String,
    pub torrent_metadata_path: Option<String>,
    pub last_error: Option<String>,
    pub imported_at: Option<String>,
    pub cleaned_at: Option<String>,
    pub reseed_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FileMapping {
    original_rel: String,
    library_rel: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SeedPolicy {
    pub indexer_id: String,
    pub min_seed_time_minutes: i64,
    pub min_ratio: f64,
    pub requirement_mode: String,
    pub cleanup_mode: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct SeedPolicyInput {
    pub min_seed_time_minutes: i64,
    pub min_ratio: f64,
    pub requirement_mode: String,
    pub cleanup_mode: String,
}

#[derive(Debug, Serialize)]
pub struct ReseedResponse {
    pub status: &'static str,
    pub path: String,
}

pub fn spawn_import_scheduler(state: AppState) {
    tokio::spawn(async move {
        loop {
            if let Err(error) = run_cycle(&state).await {
                tracing::error!(error = %error.1, "importer cycle failed");
            }
            sleep(Duration::from_secs(10)).await;
        }
    });
}

pub async fn list_imports(
    State(state): State<AppState>,
) -> Result<Json<Vec<DownloadJob>>, (StatusCode, String)> {
    let rows=sqlx::query_as::<_,DownloadJob>(r#"
        SELECT id,media_type,media_id,release_title,indexer_id,indexer_name,category,qbittorrent_tags,
               season_number,episode_number,is_season_pack,qb_hash,status,source_path,library_path,import_method,file_mappings_json,torrent_metadata_path,last_error,
               imported_at,cleaned_at,reseed_count,created_at,updated_at
        FROM download_jobs ORDER BY id DESC LIMIT 500
    "#).fetch_all(&state.db).await.map_err(internal)?;
    Ok(Json(rows))
}

#[derive(Debug, Deserialize)]
pub struct DeleteImportsRequest {
    pub ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
pub struct DeleteImportsResponse {
    pub deleted: u64,
}

/// Removes only obsolete Oberiz records. It never removes qBittorrent torrents or media files.
pub async fn delete_imports(
    State(state): State<AppState>,
    Json(payload): Json<DeleteImportsRequest>,
) -> Result<Json<DeleteImportsResponse>, (StatusCode, String)> {
    let ids = payload
        .ids
        .into_iter()
        .filter(|id| *id > 0)
        .collect::<Vec<_>>();
    if ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Select at least one record".into()));
    }
    let mut query = QueryBuilder::new(
        "DELETE FROM download_jobs WHERE id NOT IN (SELECT job_id FROM torrent_archives) AND status IN ('duplicate','error','missing','cleaned','rejected') AND id IN (",
    );
    let mut separated = query.separated(",");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");
    let result = query.build().execute(&state.db).await.map_err(internal)?;
    Ok(Json(DeleteImportsResponse {
        deleted: result.rows_affected(),
    }))
}

pub async fn list_seed_policies(
    State(state): State<AppState>,
) -> Result<Json<Vec<SeedPolicy>>, (StatusCode, String)> {
    let rows=sqlx::query_as::<_,SeedPolicy>(
        "SELECT indexer_id,min_seed_time_minutes,min_ratio,requirement_mode,cleanup_mode,updated_at FROM seed_policies ORDER BY indexer_id"
    ).fetch_all(&state.db).await.map_err(internal)?;
    Ok(Json(rows))
}

pub async fn upsert_seed_policy(
    State(state): State<AppState>,
    AxumPath(indexer_id): AxumPath<String>,
    Json(input): Json<SeedPolicyInput>,
) -> Result<Json<SeedPolicy>, (StatusCode, String)> {
    let requirement = match input.requirement_mode.as_str() {
        "time" => "time",
        "ratio" => "ratio",
        "either" => "either",
        "both" => "both",
        _ => "manual",
    };
    let cleanup = match input.cleanup_mode.as_str() {
        "remove_torrent_keep_files" => "remove_torrent_keep_files",
        "remove_torrent_and_original" => "remove_torrent_keep_files",
        "never" => "never",
        _ => "manual",
    };
    sqlx::query(r#"
        INSERT INTO seed_policies(indexer_id,min_seed_time_minutes,min_ratio,requirement_mode,cleanup_mode,updated_at)
        VALUES(?,?,?,?,?,CURRENT_TIMESTAMP)
        ON CONFLICT(indexer_id) DO UPDATE SET
          min_seed_time_minutes=excluded.min_seed_time_minutes,
          min_ratio=excluded.min_ratio,
          requirement_mode=excluded.requirement_mode,
          cleanup_mode=excluded.cleanup_mode,
          updated_at=CURRENT_TIMESTAMP
    "#)
        .bind(&indexer_id).bind(input.min_seed_time_minutes.max(0)).bind(input.min_ratio.max(0.0))
        .bind(requirement).bind(cleanup)
        .execute(&state.db).await.map_err(internal)?;

    let row=sqlx::query_as::<_,SeedPolicy>(
        "SELECT indexer_id,min_seed_time_minutes,min_ratio,requirement_mode,cleanup_mode,updated_at FROM seed_policies WHERE indexer_id=?"
    ).bind(&indexer_id).fetch_one(&state.db).await.map_err(internal)?;
    Ok(Json(row))
}

pub async fn reseed(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<i64>,
) -> Result<Json<ReseedResponse>, (StatusCode, String)> {
    let _guard = MEDIA_OPERATION_GATE.lock().await;
    let job = get_job(&state, id)
        .await?
        .ok_or((StatusCode::NOT_FOUND, "Download record not found".into()))?;
    remote::restore(&state, &job).await.map(Json)
}

async fn run_cycle(state: &AppState) -> Result<(), (StatusCode, String)> {
    let torrents = state.download_client.list_torrents(state).await?;

    for torrent in &torrents {
        let job_id = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM download_jobs WHERE qb_hash=? ORDER BY id DESC LIMIT 1",
        )
        .bind(&torrent.hash)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?
        .or_else(|| tag_job_id(&torrent.tags));

        if let Some(job_id) = job_id {
            sqlx::query(r#"
                UPDATE download_jobs
                SET qb_hash=?,source_path=?,status=CASE WHEN imported_at IS NULL THEN ? ELSE status END,updated_at=CURRENT_TIMESTAMP
                WHERE id=?
            "#)
                .bind(&torrent.hash)
                .bind(&torrent.content_path)
                .bind(if torrent.progress>=0.999999{"completed"}else{"downloading"})
                .bind(job_id)
                .execute(&state.db).await.map_err(internal)?;

            let job = get_job(state, job_id).await?;
            if let Some(job) = job {
                // Recovery stays outside cleanup until the user explicitly
                // resumes it and qBittorrent reports an active seeding state.
                if job.status == "restored_stopped"
                    && matches!(
                        torrent.state.as_str(),
                        "uploading" | "stalledUP" | "forcedUP" | "queuedUP"
                    )
                {
                    sqlx::query("UPDATE download_jobs SET status='seeding',cleaned_at=NULL,updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(job.id).execute(&state.db).await.map_err(internal)?;
                }
                if torrent.progress >= 0.999999
                    && torrent.amount_left == 0
                    && job.imported_at.is_none()
                {
                    if let Err((_, error)) = remote::complete(state, &job, torrent).await {
                        sqlx::query("UPDATE download_jobs SET status='error',last_error=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                            .bind(&error).bind(job.id).execute(&state.db).await.ok();
                        history::record(
                            &state.db,
                            "import.failed",
                            &job.release_title,
                            Some(&error),
                            "error",
                        )
                        .await;
                    }
                } else if job.imported_at.is_some() && job.cleaned_at.is_none() {
                    // Not propagated with `?`: this used to abort the whole cycle
                    // on the first torrent whose cleanup failed (e.g. qBittorrent
                    // briefly unreachable), silently blocking every other
                    // torrent's import/cleanup behind it — every 10 seconds,
                    // forever, until whatever caused the one failure was fixed.
                    if let Err((_, error)) = maybe_cleanup(state, &job, torrent).await {
                        history::record(
                            &state.db,
                            "seed.cleanup_failed",
                            &job.release_title,
                            Some(&error),
                            "error",
                        )
                        .await;
                    }
                }
            }
        }
    }
    Ok(())
}

async fn maybe_cleanup(
    state: &AppState,
    job: &DownloadJob,
    torrent: &qbittorrent::QBittorrentTorrent,
) -> Result<(), (StatusCode, String)> {
    if !setting_bool(state, "import.cleanup_after_seed", true).await? {
        return Ok(());
    }
    let policy=sqlx::query_as::<_,SeedPolicy>(
        "SELECT indexer_id,min_seed_time_minutes,min_ratio,requirement_mode,cleanup_mode,updated_at FROM seed_policies WHERE indexer_id=?"
    ).bind(&job.indexer_id).fetch_optional(&state.db).await.map_err(internal)?;
    let Some(policy) = policy else {
        return Ok(());
    };

    if matches!(policy.requirement_mode.as_str(), "manual")
        || matches!(policy.cleanup_mode.as_str(), "manual" | "never")
    {
        return Ok(());
    }
    let time_ok = torrent.seeding_time >= policy.min_seed_time_minutes.max(0) * 60;
    let ratio_ok = torrent.ratio >= policy.min_ratio.max(0.0);
    let satisfied = match policy.requirement_mode.as_str() {
        "time" => time_ok,
        "ratio" => ratio_ok,
        "either" => time_ok || ratio_ok,
        "both" => time_ok && ratio_ok,
        _ => false,
    };
    if !satisfied {
        return Ok(());
    }

    let hash = job.qb_hash.as_deref().unwrap_or(&torrent.hash);
    // Sharing policies never delete library data.
    let delete_files = false;
    state
        .download_client
        .delete_torrent(state, hash, delete_files)
        .await?;
    sqlx::query("UPDATE download_jobs SET status='cleaned',cleaned_at=CURRENT_TIMESTAMP,updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(job.id).execute(&state.db).await.map_err(internal)?;
    history::record(
        &state.db,
        "seed.cleaned",
        &job.release_title,
        Some(&format!(
            "{} · ratio {:.2} · {} min",
            job.indexer_id,
            torrent.ratio,
            torrent.seeding_time / 60
        )),
        "info",
    )
    .await;
    tracing::info!(
        job_id = job.id,
        ratio = torrent.ratio,
        seed_minutes = torrent.seeding_time / 60,
        "torrent cleanup completed"
    );
    Ok(())
}

async fn media_identity(
    state: &AppState,
    media_type: &str,
    media_id: i64,
) -> Result<(String, Option<i32>), (StatusCode, String)> {
    if media_id <= 0 {
        return Ok(("Unmatched Media".into(), None));
    }
    if media_type == "movie" {
        sqlx::query_as::<_, (String, Option<i32>)>("SELECT title,year FROM movies WHERE id=?")
            .bind(media_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?
            .ok_or_else(|| {
                (
                    StatusCode::NOT_FOUND,
                    "Película asociada no encontrada".into(),
                )
            })
    } else {
        sqlx::query_as::<_, (String, Option<i32>)>("SELECT name,year FROM series WHERE id=?")
            .bind(media_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Serie asociada no encontrada".into()))
    }
}

async fn get_job(state: &AppState, id: i64) -> Result<Option<DownloadJob>, (StatusCode, String)> {
    sqlx::query_as::<_,DownloadJob>(r#"
        SELECT id,media_type,media_id,release_title,indexer_id,indexer_name,category,qbittorrent_tags,
               season_number,episode_number,is_season_pack,qb_hash,status,source_path,library_path,import_method,file_mappings_json,torrent_metadata_path,last_error,
               imported_at,cleaned_at,reseed_count,created_at,updated_at
        FROM download_jobs WHERE id=?
    "#).bind(id).fetch_optional(&state.db).await.map_err(internal)
}

fn tag_job_id(tags: &str) -> Option<i64> {
    tags.split(',').map(str::trim).find_map(|tag| {
        tag.strip_prefix("_oberiz_job_")
            .or_else(|| tag.strip_prefix("job:"))
            .and_then(|v| v.parse::<i64>().ok())
    })
}

async fn setting_bool(
    state: &AppState,
    key: &str,
    default: bool,
) -> Result<bool, (StatusCode, String)> {
    Ok(settings::get_value(&state.db, key)
        .await
        .map_err(internal)?
        .map(|v| v == "true")
        .unwrap_or(default))
}
fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
