use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{AppState, history, releases, settings};

#[derive(Debug, Deserialize)]
pub struct RescanQuery {
    pub media_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RescanResult {
    pub media_type: String,
    pub root_path: String,
    pub scanned_files: usize,
    pub matched_files: usize,
    pub unmatched_files: usize,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ScanHistory {
    pub id: i64,
    pub media_type: String,
    pub root_path: String,
    pub scanned_files: i64,
    pub matched_files: i64,
    pub unmatched_files: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
struct MovieCandidate {
    id: i64,
    title: String,
    year: Option<i32>,
}
#[derive(Clone)]
struct SeriesCandidate {
    id: i64,
    name: String,
}

pub async fn rescan(
    State(state): State<AppState>,
    Query(query): Query<RescanQuery>,
) -> Result<Json<Vec<RescanResult>>, (StatusCode, String)> {
    let target = query.media_type.as_deref().unwrap_or("all");
    if !matches!(target, "all" | "movie" | "series") {
        return Err((
            StatusCode::BAD_REQUEST,
            "media_type debe ser movie, series o all".into(),
        ));
    }
    let mut out = Vec::new();
    if target == "all" || target == "movie" {
        out.push(scan_movies(&state).await?);
    }
    if target == "all" || target == "series" {
        out.push(scan_series(&state).await?);
    }
    Ok(Json(out))
}

#[derive(Debug, Serialize)]
pub struct LibrarySummary {
    pub movie_files: i64,
    pub series_files: i64,
    pub movie_bytes: i64,
    pub series_bytes: i64,
    pub total_bytes: i64,
}

pub async fn summary(
    State(state): State<AppState>,
) -> Result<Json<LibrarySummary>, (StatusCode, String)> {
    let movie_files = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM media_files WHERE media_type='movie' AND file_exists=1",
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;
    let series_files = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM media_files WHERE media_type='series' AND file_exists=1",
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;
    let movie_bytes=sqlx::query_scalar::<_,i64>(
        "SELECT COALESCE(SUM(size_bytes),0) FROM media_files WHERE media_type='movie' AND file_exists=1"
    ).fetch_one(&state.db).await.map_err(internal)?;
    let series_bytes=sqlx::query_scalar::<_,i64>(
        "SELECT COALESCE(SUM(size_bytes),0) FROM media_files WHERE media_type='series' AND file_exists=1"
    ).fetch_one(&state.db).await.map_err(internal)?;
    Ok(Json(LibrarySummary {
        movie_files,
        series_files,
        movie_bytes,
        series_bytes,
        total_bytes: movie_bytes + series_bytes,
    }))
}

pub async fn scan_history(
    State(state): State<AppState>,
) -> Result<Json<Vec<ScanHistory>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, ScanHistory>(
        r#"
        SELECT id,media_type,root_path,scanned_files,matched_files,unmatched_files,
               started_at,completed_at,error
        FROM library_scans ORDER BY id DESC LIMIT 20
    "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;
    Ok(Json(rows))
}

async fn scan_movies(state: &AppState) -> Result<RescanResult, (StatusCode, String)> {
    let root = settings::get_value(&state.db, "paths.movies")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if root.trim().is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "Configura primero la ruta Movies".into(),
        ));
    }
    let root_path = PathBuf::from(&root);
    if !root_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("La ruta Movies no existe: {}", root_path.display()),
        ));
    }

    let scan_id = start_scan(state, "movie", &root).await?;
    let files = collect_video_files(&root_path).map_err(fs_error)?;
    let movies =
        sqlx::query_as::<_, (i64, String, Option<i32>)>("SELECT id,title,year FROM movies")
            .fetch_all(&state.db)
            .await
            .map_err(internal)?
            .into_iter()
            .map(|x| MovieCandidate {
                id: x.0,
                title: x.1,
                year: x.2,
            })
            .collect::<Vec<_>>();

    sqlx::query("UPDATE media_files SET file_exists=0 WHERE media_type='movie'")
        .execute(&state.db)
        .await
        .map_err(internal)?;

    let mut matched = 0usize;
    for file in &files {
        let haystack = normalized(&file.to_string_lossy());
        let mut best: Option<&MovieCandidate> = None;
        for movie in &movies {
            let title = normalized(&movie.title);
            if title.len() < 2 || !haystack.contains(&title) {
                continue;
            }
            if let Some(year) = movie.year {
                let year_text = year.to_string();
                if haystack.chars().any(|c| c.is_ascii_digit()) && !haystack.contains(&year_text) {
                    // Year mismatch is a weak signal, not a hard rejection when the path has no obvious year.
                }
            }
            if best
                .as_ref()
                .map(|b| normalized(&b.title).len())
                .unwrap_or(0)
                < title.len()
            {
                best = Some(movie);
            }
        }
        if let Some(movie) = best {
            upsert_media_file(state, "movie", movie.id, file, "scan").await?;
            matched += 1;
        }
    }
    finish_scan(state, scan_id, files.len(), matched, None).await?;
    history::record(
        &state.db,
        "library.rescan.movies",
        "Movies",
        Some(&format!("{} matched / {} scanned", matched, files.len())),
        "info",
    )
    .await;
    Ok(RescanResult {
        media_type: "movie".into(),
        root_path: root,
        scanned_files: files.len(),
        matched_files: matched,
        unmatched_files: files.len().saturating_sub(matched),
    })
}

async fn scan_series(state: &AppState) -> Result<RescanResult, (StatusCode, String)> {
    let root = settings::get_value(&state.db, "paths.series")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if root.trim().is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "Configura primero la ruta Series".into(),
        ));
    }
    let root_path = PathBuf::from(&root);
    if !root_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("La ruta Series no existe: {}", root_path.display()),
        ));
    }

    let scan_id = start_scan(state, "series", &root).await?;
    let files = collect_video_files(&root_path).map_err(fs_error)?;
    let series = sqlx::query_as::<_, (i64, String)>("SELECT id,name FROM series")
        .fetch_all(&state.db)
        .await
        .map_err(internal)?
        .into_iter()
        .map(|x| SeriesCandidate { id: x.0, name: x.1 })
        .collect::<Vec<_>>();

    sqlx::query("UPDATE media_files SET file_exists=0 WHERE media_type='series'")
        .execute(&state.db)
        .await
        .map_err(internal)?;
    sqlx::query("UPDATE series_episodes SET has_file=0")
        .execute(&state.db)
        .await
        .map_err(internal)?;

    let mut matched = 0usize;
    for file in &files {
        let Some((season, first, last)) =
            parse_episode_span(file.file_name().and_then(|x| x.to_str()).unwrap_or(""))
        else {
            continue;
        };
        let haystack = normalized(&file.to_string_lossy());
        let mut best: Option<&SeriesCandidate> = None;
        for item in &series {
            let name = normalized(&item.name);
            if name.len() < 2 || !haystack.contains(&name) {
                continue;
            }
            if best
                .as_ref()
                .map(|b| normalized(&b.name).len())
                .unwrap_or(0)
                < name.len()
            {
                best = Some(item);
            }
        }
        let Some(item) = best else {
            continue;
        };
        let media_file_id = upsert_media_file(state, "series", item.id, file, "scan").await?;
        let last = last.unwrap_or(first).max(first);
        let mut linked = false;
        for episode in first..=last {
            if let Some(ep_id)=sqlx::query_scalar::<_,i64>(
                "SELECT id FROM series_episodes WHERE series_id=? AND season_number=? AND episode_number=?"
            ).bind(item.id).bind(season).bind(episode).fetch_optional(&state.db).await.map_err(internal)? {
                sqlx::query("UPDATE series_episodes SET has_file=1,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                    .bind(ep_id).execute(&state.db).await.map_err(internal)?;
                sqlx::query(r#"INSERT INTO episode_files(episode_id,media_file_id) VALUES(?,?)
                    ON CONFLICT(episode_id,media_file_id) DO NOTHING"#)
                    .bind(ep_id).bind(media_file_id).execute(&state.db).await.map_err(internal)?;
                linked=true;
            }
        }
        if linked {
            matched += 1;
        }
    }
    finish_scan(state, scan_id, files.len(), matched, None).await?;
    history::record(
        &state.db,
        "library.rescan.series",
        "Series",
        Some(&format!("{} matched / {} scanned", matched, files.len())),
        "info",
    )
    .await;
    Ok(RescanResult {
        media_type: "series".into(),
        root_path: root,
        scanned_files: files.len(),
        matched_files: matched,
        unmatched_files: files.len().saturating_sub(matched),
    })
}

async fn upsert_media_file(
    state: &AppState,
    media_type: &str,
    media_id: i64,
    path: &Path,
    discovered_by: &str,
) -> Result<i64, (StatusCode, String)> {
    let name = path.file_name().and_then(|x| x.to_str()).unwrap_or("");
    let parsed = releases::parse(name);
    let size = fs::metadata(path).ok().map(|m| m.len() as i64);
    let quality_json = serde_json::to_string(&parsed).unwrap_or_else(|_| "{}".into());
    sqlx::query_scalar::<_,i64>(r#"
        INSERT INTO media_files(
          media_type,media_id,download_job_id,path,size_bytes,quality_json,source_release,
          resolution,source,codec,hdr,audio,language,quality_score,file_exists,discovered_by,verified_at
        ) VALUES(?,?,NULL,?,?,?,?,?,?,?,?,?,?,?,1,?,CURRENT_TIMESTAMP)
        ON CONFLICT(path) DO UPDATE SET
          media_type=excluded.media_type,media_id=excluded.media_id,size_bytes=excluded.size_bytes,
          quality_json=excluded.quality_json,resolution=excluded.resolution,source=excluded.source,
          codec=excluded.codec,hdr=excluded.hdr,audio=excluded.audio,language=excluded.language,
          quality_score=excluded.quality_score,file_exists=1,verified_at=CURRENT_TIMESTAMP
        RETURNING id
    "#)
        .bind(media_type).bind(media_id).bind(path.to_string_lossy().to_string()).bind(size)
        .bind(&quality_json).bind(name)
        .bind(parsed.resolution).bind(parsed.source).bind(parsed.codec).bind(parsed.hdr)
        .bind(parsed.audio).bind(parsed.language).bind(parsed.score).bind(discovered_by)
        .fetch_one(&state.db).await.map_err(internal)
}

async fn start_scan(
    state: &AppState,
    media_type: &str,
    root: &str,
) -> Result<i64, (StatusCode, String)> {
    sqlx::query_scalar::<_, i64>(
        "INSERT INTO library_scans(media_type,root_path) VALUES(?,?) RETURNING id",
    )
    .bind(media_type)
    .bind(root)
    .fetch_one(&state.db)
    .await
    .map_err(internal)
}
async fn finish_scan(
    state: &AppState,
    id: i64,
    scanned: usize,
    matched: usize,
    error: Option<&str>,
) -> Result<(), (StatusCode, String)> {
    sqlx::query("UPDATE library_scans SET scanned_files=?,matched_files=?,unmatched_files=?,completed_at=CURRENT_TIMESTAMP,error=? WHERE id=?")
        .bind(scanned as i64).bind(matched as i64).bind(scanned.saturating_sub(matched) as i64).bind(error).bind(id)
        .execute(&state.db).await.map_err(internal)?;
    Ok(())
}

fn collect_video_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_recursive(root, &mut out)?;
    Ok(out)
}
fn collect_recursive(path: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_file() {
        if is_video(path) {
            out.push(path.to_path_buf());
        }
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_recursive(&p, out)?;
        } else if is_video(&p) {
            out.push(p);
        }
    }
    Ok(())
}
fn is_video(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "mkv" | "mp4" | "avi" | "mov" | "m4v" | "ts" | "m2ts" | "webm"
    )
}
fn normalized(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn parse_episode_span(name: &str) -> Option<(i32, i32, Option<i32>)> {
    let patterns = [
        r"(?i)S(\d{1,2})E(\d{1,3})(?:[-_. ]?E?(\d{1,3}))?",
        r"(?i)(\d{1,2})x(\d{1,3})(?:[-_. ]?(\d{1,3}))?",
    ];
    for pattern in patterns {
        let re = regex::Regex::new(pattern).ok()?;
        if let Some(caps) = re.captures(name) {
            return Some((
                caps.get(1)?.as_str().parse().ok()?,
                caps.get(2)?.as_str().parse().ok()?,
                caps.get(3).and_then(|m| m.as_str().parse().ok()),
            ));
        }
    }
    None
}
fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
fn fs_error(e: std::io::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
