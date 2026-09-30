mod naming;
mod transfer;

use axum::{
    Json,
    extract::{Path as AxumPath, State},
    http::StatusCode,
};
use naming::{
    parse_absolute_episode, parse_episode_numbers, render_series_template, render_template,
    sanitize_name,
};
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};
use tokio::time::{Duration, sleep};
use transfer::{collect_media_files, transfer_file};

use crate::{AppState, history, qbittorrent, releases, settings};

/// Marks jobs missing only after qBittorrent answered successfully.
/// This also clears an old queued job that never received a qBittorrent hash, so a failed
/// hand-off cannot indefinitely stop automation from trying the media again.
pub(crate) async fn reconcile_missing_torrents(
    state: &AppState,
) -> Result<usize, (StatusCode, String)> {
    let torrents = state.download_client.list_torrents(state).await?;
    let present: HashSet<String> = torrents.into_iter().map(|torrent| torrent.hash).collect();
    let jobs = sqlx::query_as::<_, (i64, String, Option<String>)>(
        r#"
        SELECT id,release_title,qb_hash FROM download_jobs
        WHERE status IN ('queued','downloading','completed','seeding')
          AND (
            (qb_hash IS NOT NULL AND qb_hash<>'')
            OR (
              status='queued'
              AND (qb_hash IS NULL OR qb_hash='')
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
            if setting_bool(&state, "import.enabled", true)
                .await
                .unwrap_or(true)
                && let Err(error) = run_cycle(&state).await
            {
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
        "DELETE FROM download_jobs WHERE status IN ('duplicate','error','missing','cleaned') AND id IN (",
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
        "remove_torrent_and_original" => "remove_torrent_and_original",
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
    let job = get_job(&state, id)
        .await?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Import job no encontrado".into()))?;
    let metadata = job.torrent_metadata_path.clone().ok_or_else(|| {
        (
            StatusCode::PRECONDITION_REQUIRED,
            "No hay .torrent guardado para reseed".into(),
        )
    })?;
    let library = job.library_path.clone().ok_or_else(|| {
        (
            StatusCode::PRECONDITION_REQUIRED,
            "No hay ruta de biblioteca registrada".into(),
        )
    })?;

    let bytes = fs::read(&metadata).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("No se pudo leer {metadata}: {e}"),
        )
    })?;
    let mappings: Vec<FileMapping> =
        serde_json::from_str(&job.file_mappings_json).unwrap_or_default();
    if mappings.is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "No existe mapping de archivos para reseed".into(),
        ));
    }

    let configured = settings::get_value(&state.db, "paths.reseed")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    let downloads = settings::get_value(&state.db, "paths.downloads")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    let base = if !configured.trim().is_empty() {
        PathBuf::from(configured)
    } else if !downloads.trim().is_empty() {
        PathBuf::from(downloads).join("reseed")
    } else {
        PathBuf::from("./reseed")
    };
    refuse_system_directory(&base)?;
    fs::create_dir_all(&base).map_err(fs_error)?;

    let library_root = PathBuf::from(&library);
    refuse_system_directory(&library_root)?;
    for mapping in &mappings {
        let src = resolve_within(&library_root, &mapping.library_rel)?;
        let dst = resolve_within(&base, &mapping.original_rel)?;
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(fs_error)?;
        }
        if dst.exists() {
            let _ = fs::remove_file(&dst);
        }
        if fs::hard_link(&src, &dst).is_err() {
            fs::copy(&src, &dst).map_err(fs_error)?;
        }
    }

    let base_string = base.to_string_lossy().to_string();
    let qb_hash = state
        .download_client
        .add_torrent_file(
            &state,
            bytes,
            &job.category,
            &job.release_title,
            &job.qbittorrent_tags,
            Some(job.id),
            Some(&base_string),
            true,
        )
        .await?;
    let _ =
        sqlx::query("UPDATE download_jobs SET qb_hash=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(&qb_hash)
            .bind(job.id)
            .execute(&state.db)
            .await;

    sqlx::query("UPDATE download_jobs SET reseed_count=reseed_count+1,updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(id).execute(&state.db).await.map_err(internal)?;
    history::record(
        &state.db,
        "reseed.started",
        &job.release_title,
        Some(&base_string),
        "info",
    )
    .await;
    Ok(Json(ReseedResponse {
        status: "ok",
        path: base_string,
    }))
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
                if torrent.progress >= 0.999999 && job.imported_at.is_none() {
                    if let Err((_, error)) = import_job(state, &job, torrent).await {
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

async fn import_job(
    state: &AppState,
    job: &DownloadJob,
    torrent: &qbittorrent::QBittorrentTorrent,
) -> Result<(), (StatusCode, String)> {
    let source = PathBuf::from(&torrent.content_path);
    if !source.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("La ruta descargada no existe: {}", source.display()),
        ));
    }

    let root_setting = if job.media_type == "movie" {
        "paths.movies"
    } else {
        "paths.series"
    };
    let library_base = settings::get_value(&state.db, root_setting)
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if library_base.trim().is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            format!("Configura {} en Settings", root_setting),
        ));
    }

    let (title, year) = media_identity(state, &job.media_type, job.media_id).await?;
    let folder_name = if let Some(year) = year {
        format!("{} ({})", sanitize_name(&title), year)
    } else {
        sanitize_name(&title)
    };
    let library_root = PathBuf::from(library_base).join(folder_name);
    refuse_system_directory(&library_root)?;
    fs::create_dir_all(&library_root).map_err(fs_error)?;

    let method = settings::get_value(&state.db, "import.method")
        .await
        .map_err(internal)?
        .unwrap_or_else(|| "auto".into());
    let rename = setting_bool(state, "import.rename_enabled", true).await?;
    let mappings = copy_payload(
        state,
        job,
        torrent,
        &source,
        &library_root,
        &method,
        rename,
        &title,
        year,
    )
    .await?;

    let metadata_path = if setting_bool(state, "import.keep_reseed_metadata", true).await? {
        save_torrent_metadata(state, job, torrent).await.ok()
    } else {
        None
    };

    let parsed = releases::parse(&job.release_title);
    let quality_json = serde_json::to_string(&parsed).unwrap_or_else(|_| "{}".into());

    for mapping in &mappings {
        let path = library_root.join(&mapping.library_rel);
        let size = fs::metadata(&path).ok().map(|m| m.len() as i64);
        let path_string = path.to_string_lossy().to_string();
        let media_file_id=sqlx::query_scalar::<_,i64>(r#"
            INSERT INTO media_files(
              media_type,media_id,download_job_id,path,size_bytes,quality_json,source_release,
              resolution,source,codec,hdr,audio,language,quality_score,file_exists,discovered_by,verified_at
            ) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,1,'import',CURRENT_TIMESTAMP)
            ON CONFLICT(path) DO UPDATE SET
              download_job_id=excluded.download_job_id,size_bytes=excluded.size_bytes,
              quality_json=excluded.quality_json,source_release=excluded.source_release,
              resolution=excluded.resolution,source=excluded.source,codec=excluded.codec,hdr=excluded.hdr,
              audio=excluded.audio,language=excluded.language,quality_score=excluded.quality_score,
              file_exists=1,verified_at=CURRENT_TIMESTAMP
            RETURNING id
        "#)
            .bind(&job.media_type).bind(job.media_id).bind(job.id)
            .bind(&path_string).bind(size).bind(&quality_json).bind(&job.release_title)
            .bind(&parsed.resolution).bind(&parsed.source).bind(&parsed.codec).bind(&parsed.hdr)
            .bind(&parsed.audio).bind(&parsed.language).bind(parsed.score)
            .fetch_one(&state.db).await.map_err(internal)?;

        if job.media_type == "series" {
            link_series_file(state, job, media_file_id, &mapping.library_rel).await?;
        }
    }

    if job.media_type == "series"
        && job.episode_number.is_none()
        && let Some(season) = job.season_number
    {
        // This job was grabbed as a whole-season pack (no single episode
        // number), so every AIRED episode in `season` belongs to it even if
        // one file's name didn't match the SxxExx/absolute-episode patterns
        // above (extras, a differently-named bonus episode, an unusual scene
        // convention...). Without this, that one episode would stay
        // has_file=0 forever, automation would keep treating the season as
        // "still missing something", and — once the original job's seeding
        // was cleaned up — nothing would stop it from grabbing the season
        // again from scratch, which is exactly the bug this fixes. Excluding
        // unaired episodes matters just as much: a pack only ever contains
        // what had already aired when it was released, so marking a future
        // episode as already-on-disk would hide it from automation forever.
        sqlx::query(
            "UPDATE series_episodes SET has_file=1,updated_at=CURRENT_TIMESTAMP WHERE series_id=? AND season_number=? AND has_file=0 AND (air_date IS NULL OR air_date<=date('now','localtime'))",
        )
        .bind(job.media_id)
        .bind(season)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    }

    let source_string = source.to_string_lossy().to_string();
    let library_string = library_root.to_string_lossy().to_string();
    let mappings_json = serde_json::to_string(&mappings).unwrap_or_else(|_| "[]".into());
    sqlx::query(r#"
        UPDATE download_jobs SET
          status='seeding',source_path=?,library_path=?,import_method=?,
          file_mappings_json=?,torrent_metadata_path=?,imported_at=CURRENT_TIMESTAMP,last_error=NULL,updated_at=CURRENT_TIMESTAMP
        WHERE id=?
    "#)
        .bind(&source_string)
        .bind(&library_string)
        .bind(method_used(&method,&source,&library_root))
        .bind(&mappings_json)
        .bind(metadata_path)
        .bind(job.id).execute(&state.db).await.map_err(internal)?;

    history::record(
        &state.db,
        "import.completed",
        &job.release_title,
        Some(&format!(
            "{} -> {}",
            source.display(),
            library_root.display()
        )),
        "info",
    )
    .await;
    tracing::info!(
        job_id = job.id,
        source = %source.display(),
        destination = %library_root.display(),
        "import completed"
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn copy_payload(
    state: &AppState,
    job: &DownloadJob,
    torrent: &qbittorrent::QBittorrentTorrent,
    source: &Path,
    library_root: &Path,
    method: &str,
    rename: bool,
    title: &str,
    year: Option<i32>,
) -> Result<Vec<FileMapping>, (StatusCode, String)> {
    let mut files = Vec::<PathBuf>::new();
    collect_media_files(source, &mut files).map_err(fs_error)?;
    if files.is_empty() {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            "No se encontraron archivos multimedia para importar".into(),
        ));
    }

    let save_path = PathBuf::from(&torrent.save_path);
    let parsed = releases::parse(&job.release_title);
    let mut mappings = Vec::new();

    for (index, src) in files.iter().enumerate() {
        let original_rel = src
            .strip_prefix(&save_path)
            .unwrap_or(src)
            .to_string_lossy()
            .to_string();
        let ext = src.extension().and_then(|x| x.to_str()).unwrap_or("");
        let library_rel = if job.media_type == "series" {
            let detected =
                parse_episode_numbers(src.file_name().and_then(|x| x.to_str()).unwrap_or(""));
            let season = detected.map(|x| x.0).or(job.season_number);
            let episode = detected.map(|x| x.1).or(job.episode_number);
            let fallback_rel = if source.is_file() {
                src.file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or("media")
                    .to_string()
            } else {
                src.strip_prefix(source)
                    .unwrap_or(src)
                    .to_string_lossy()
                    .to_string()
            };
            if rename {
                if let (Some(season), Some(episode)) = (season, episode) {
                    let template = settings::get_value(&state.db, "import.series_template")
                        .await
                        .map_err(internal)?
                        .unwrap_or_else(|| {
                            "{Title} - S{Season:00}E{Episode:00} - {EpisodeTitle}".into()
                        });
                    let episode_title=sqlx::query_scalar::<_,String>(
                        "SELECT name FROM series_episodes WHERE series_id=? AND season_number=? AND episode_number=?"
                    ).bind(job.media_id).bind(season).bind(episode)
                        .fetch_optional(&state.db).await.map_err(internal)?.unwrap_or_default();
                    let base = render_series_template(
                        &template,
                        title,
                        year,
                        &parsed,
                        season,
                        episode,
                        &episode_title,
                    );
                    let file = if ext.is_empty() {
                        sanitize_name(&base)
                    } else {
                        format!("{}.{}", sanitize_name(&base), ext)
                    };
                    format!("Season {:02}/{}", season, file)
                } else {
                    fallback_rel
                }
            } else {
                fallback_rel
            }
        } else if source.is_file() || files.len() == 1 {
            if rename {
                let template = settings::get_value(&state.db, "import.movie_template")
                    .await
                    .map_err(internal)?
                    .unwrap_or_else(|| "{Title} ({Year}) - {Resolution} {Source} {Codec}".into());
                let base = render_template(&template, title, year, &parsed);
                if ext.is_empty() {
                    sanitize_name(&base)
                } else {
                    format!("{}.{}", sanitize_name(&base), ext)
                }
            } else {
                src.file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or("media")
                    .to_string()
            }
        } else {
            src.strip_prefix(source)
                .unwrap_or(src)
                .to_string_lossy()
                .to_string()
        };

        let dst = library_root.join(&library_rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(fs_error)?;
        }
        if dst.exists() {
            let src_len = fs::metadata(src).map_err(fs_error)?.len();
            let dst_len = fs::metadata(&dst).map_err(fs_error)?.len();
            if src_len == dst_len {
                mappings.push(FileMapping {
                    original_rel,
                    library_rel,
                });
                continue;
            }
            return Err((
                StatusCode::CONFLICT,
                format!("Ya existe un archivo distinto: {}", dst.display()),
            ));
        }

        transfer_file(src, &dst, method).map_err(fs_error)?;
        mappings.push(FileMapping {
            original_rel,
            library_rel,
        });
        let _ = index;
    }
    Ok(mappings)
}

async fn save_torrent_metadata(
    state: &AppState,
    job: &DownloadJob,
    torrent: &qbittorrent::QBittorrentTorrent,
) -> Result<String, (StatusCode, String)> {
    let hash = if torrent.hash.is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "qBittorrent hash no disponible".into(),
        ));
    } else {
        &torrent.hash
    };
    let bytes = state
        .download_client
        .export_torrent_file(state, hash)
        .await?;
    let base = settings::get_value(&state.db, "import.torrent_metadata_path")
        .await
        .map_err(internal)?
        .unwrap_or_else(settings::default_torrent_metadata_path);
    fs::create_dir_all(&base).map_err(fs_error)?;
    let path = PathBuf::from(base).join(format!("{}-{}.torrent", job.id, hash));
    fs::write(&path, bytes).map_err(fs_error)?;
    Ok(path.to_string_lossy().to_string())
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
    let delete_files = policy.cleanup_mode == "remove_torrent_and_original";
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

async fn link_series_file(
    state: &AppState,
    job: &DownloadJob,
    media_file_id: i64,
    library_rel: &str,
) -> Result<(), (StatusCode, String)> {
    let detected = parse_episode_numbers(library_rel).or_else(|| {
        job.season_number
            .zip(job.episode_number)
            .map(|(s, e)| (s, e, None))
    });

    let detected = if detected.is_some() {
        detected
    } else if let Some(absolute) = parse_absolute_episode(library_rel) {
        let offset = (absolute - 1).max(0) as i64;
        sqlx::query_as::<_,(i32,i32)>(
            "SELECT season_number,episode_number FROM series_episodes WHERE series_id=? AND season_number>0 ORDER BY season_number,episode_number LIMIT 1 OFFSET ?"
        ).bind(job.media_id).bind(offset).fetch_optional(&state.db).await.map_err(internal)?
            .map(|(s,e)|(s,e,None))
    } else {
        None
    };

    let Some((season, first, last)) = detected else {
        return Ok(());
    };
    let last = last.unwrap_or(first).max(first);
    for episode in first..=last {
        if let Some(episode_id)=sqlx::query_scalar::<_,i64>(
            "SELECT id FROM series_episodes WHERE series_id=? AND season_number=? AND episode_number=?"
        ).bind(job.media_id).bind(season).bind(episode)
            .fetch_optional(&state.db).await.map_err(internal)? {
            sqlx::query("UPDATE series_episodes SET has_file=1,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                .bind(episode_id).execute(&state.db).await.map_err(internal)?;
            sqlx::query(r#"
                INSERT INTO episode_files(episode_id,media_file_id) VALUES(?,?)
                ON CONFLICT(episode_id,media_file_id) DO NOTHING
            "#).bind(episode_id).bind(media_file_id).execute(&state.db).await.map_err(internal)?;
        }
    }
    Ok(())
}

fn method_used(method: &str, _source: &Path, _library: &Path) -> String {
    // In auto mode the actual per-file result can be a mixture only in pathological cases.
    // We report "auto" and preserve the exact mappings.
    method.to_string()
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
fn fs_error(e: std::io::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

/// Refuses a small denylist of well-known OS-critical directories as an
/// import/reseed destination or library root, regardless of how the path got
/// configured. `paths.movies`/`paths.series`/`paths.reseed` are ordinary
/// settings rows with no format Oberiz could reject as "clearly wrong" for a
/// media library — but a `.sqlite3` restored from a manipulated backup only
/// has its table *names* checked against the current schema (see
/// backups::restore_backup), not the values inside them, so those settings
/// could just as easily come from an attacker as from the person configuring
/// Settings. No legitimate media library is ever inside one of these.
fn refuse_system_directory(path: &Path) -> Result<(), (StatusCode, String)> {
    let normalized = path.to_string_lossy().to_lowercase().replace('\\', "/");
    const DENYLIST: &[&str] = &[
        "c:/windows",
        "c:/program files",
        "c:/program files (x86)",
        "c:/programdata",
        "/etc",
        "/bin",
        "/sbin",
        "/usr",
        "/boot",
        "/sys",
        "/proc",
        "/lib",
        "/lib64",
        "/root",
    ];
    let is_root = path.parent().is_none();
    let is_denied = DENYLIST
        .iter()
        .any(|prefix| normalized == *prefix || normalized.starts_with(&format!("{prefix}/")));
    if is_root || is_denied {
        return Err((
            StatusCode::FORBIDDEN,
            format!(
                "Refusing to use {} as a library/import path — it looks like an OS-critical directory.",
                path.display()
            ),
        ));
    }
    Ok(())
}

/// Joins `relative` onto `base` and refuses the result if `..` segments would
/// have walked it outside `base` — a relative path is resolved lexically
/// (component by component) rather than trusted as-is, since a `library_rel`/
/// `original_rel` this permissive about traversal ultimately comes from
/// `file_mappings_json` in the database. A `.sqlite3` restored from a
/// manipulated backup only has its table *names* checked against the current
/// schema (see backups::restore_backup), not the values inside them, so a
/// crafted mapping like `../../../etc/passwd` must not be able to turn a
/// reseed into reading (and re-sharing over BitTorrent) or deleting an
/// arbitrary file outside the job's own library/reseed folder.
fn resolve_within(base: &Path, relative: &str) -> Result<PathBuf, (StatusCode, String)> {
    let mut normalized = PathBuf::new();
    for component in base.join(relative).components() {
        match component {
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::CurDir => {}
            other => normalized.push(other),
        }
    }
    if normalized.starts_with(base) {
        Ok(normalized)
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            format!("Ruta fuera de la carpeta esperada: {relative}"),
        ))
    }
}

#[cfg(test)]
mod resolve_within_tests {
    use super::resolve_within;
    use std::path::Path;

    #[test]
    fn keeps_a_well_behaved_relative_path() {
        let base = Path::new("/library/Movie (2024)");
        let resolved = resolve_within(base, "Movie.2024.1080p.mkv").unwrap();
        assert_eq!(resolved, base.join("Movie.2024.1080p.mkv"));
    }

    #[test]
    fn rejects_a_traversal_that_escapes_the_base() {
        let base = Path::new("/library/Movie (2024)");
        assert!(resolve_within(base, "../../../etc/passwd").is_err());
    }

    #[test]
    fn rejects_traversal_hidden_inside_a_deeper_relative_path() {
        let base = Path::new("/library/Movie (2024)");
        assert!(resolve_within(base, "extras/../../../../etc/passwd").is_err());
    }
}

#[cfg(test)]
mod refuse_system_directory_tests {
    use super::refuse_system_directory;
    use std::path::Path;

    #[test]
    fn allows_an_ordinary_media_library_path() {
        assert!(refuse_system_directory(Path::new("/media/Movies")).is_ok());
        assert!(refuse_system_directory(Path::new(r"D:\Media\Series")).is_ok());
    }

    #[test]
    fn refuses_well_known_os_directories() {
        assert!(refuse_system_directory(Path::new("/etc")).is_err());
        assert!(refuse_system_directory(Path::new("/etc/oberiz")).is_err());
        assert!(refuse_system_directory(Path::new(r"C:\Windows\System32")).is_err());
        assert!(refuse_system_directory(Path::new("/")).is_err());
    }
}
