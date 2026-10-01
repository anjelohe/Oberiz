use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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
    // A mount point that exists but is empty (a network share that hasn't
    // finished mounting yet, a NAS blip) would otherwise look identical to a
    // library with nothing left in it — and the unconditional file_exists=0
    // reset below would then tell automation every movie is missing and
    // re-download the entire library. Refuse instead of guessing.
    if files.is_empty() {
        let message = "No video files found under the configured Movies path. Refusing to mark the library as missing — check that the path is mounted correctly.".to_string();
        finish_scan(state, scan_id, 0, 0, Some(&message)).await?;
        return Err((StatusCode::CONFLICT, message));
    }
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

    // The reset and every rebuilt row commit as one transaction: other
    // connections (automation, RSS, importer) see either the complete
    // pre-scan state or the complete post-scan state, never the empty window
    // in between that `file_exists=0` alone would otherwise expose while the
    // loop below is still running — and a crash or cancellation mid-scan
    // rolls back to the pre-scan state instead of leaving it half-reset.
    // A path already linked to a movie stays linked to that movie: fresh
    // substring matching below is a weak heuristic (two unrelated titles,
    // remakes, a title that's a substring of another) and re-running it on
    // every rescan could otherwise flip an already-correct association —
    // silently reassigning an imported file to the wrong movie and leaving
    // the real one to be re-downloaded. Only a path with no prior
    // association is a candidate for (re-)matching.
    let known_paths: HashMap<String, i64> = sqlx::query_as::<_, (String, i64)>(
        "SELECT path,media_id FROM media_files WHERE media_type='movie'",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?
    .into_iter()
    .collect();

    let mut tx = state.db.begin().await.map_err(internal)?;
    sqlx::query("UPDATE media_files SET file_exists=0 WHERE media_type='movie'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let mut matched = 0usize;
    for file in &files {
        let path_key = file.to_string_lossy().to_string();
        if let Some(&known_id) = known_paths.get(&path_key)
            && movies.iter().any(|m| m.id == known_id)
        {
            upsert_media_file(&mut tx, "movie", known_id, file, "scan").await?;
            matched += 1;
            continue;
        }

        let haystack = normalized(&file.to_string_lossy());
        let haystack_has_year = haystack
            .as_bytes()
            .windows(4)
            .any(|w| w.iter().all(u8::is_ascii_digit));
        let mut best: Option<(&MovieCandidate, bool)> = None;
        for movie in &movies {
            let title = normalized(&movie.title);
            if title.len() < 2 || !haystack.contains(&title) {
                continue;
            }
            let year_matches = movie
                .year
                .is_some_and(|year| haystack.contains(&year.to_string()));
            // An explicit year in the path is a strong, deliberate signal
            // (the user or scene group named it that way) that should win
            // over a same/shorter-titled candidate from a different year —
            // exactly the remake case (a 1982 and a 2011 "The Thing").
            let better = match best {
                None => true,
                Some((current, current_year_matches)) => {
                    if haystack_has_year && year_matches != current_year_matches {
                        year_matches
                    } else {
                        normalized(&current.title).len() < title.len()
                    }
                }
            };
            if better {
                best = Some((movie, year_matches));
            }
        }
        if let Some((movie, _)) = best {
            upsert_media_file(&mut tx, "movie", movie.id, file, "scan").await?;
            matched += 1;
        }
    }
    tx.commit().await.map_err(internal)?;
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
    // See scan_movies: an empty-but-present mount point must not be read as
    // "the whole library is gone", or the unconditional resets below would
    // make automation re-download every monitored series from scratch.
    if files.is_empty() {
        let message = "No video files found under the configured Series path. Refusing to mark the library as missing — check that the path is mounted correctly.".to_string();
        finish_scan(state, scan_id, 0, 0, Some(&message)).await?;
        return Err((StatusCode::CONFLICT, message));
    }
    let series = sqlx::query_as::<_, (i64, String)>("SELECT id,name FROM series")
        .fetch_all(&state.db)
        .await
        .map_err(internal)?
        .into_iter()
        .map(|x| SeriesCandidate { id: x.0, name: x.1 })
        .collect::<Vec<_>>();

    // See scan_movies for why this is one transaction: without it, other
    // connections could observe the library mid-reset (everything marked
    // missing) while this loop is still rebuilding it, and a crash midway
    // would leave that half-reset state persisted instead of rolling back.
    let mut tx = state.db.begin().await.map_err(internal)?;
    sqlx::query("UPDATE media_files SET file_exists=0 WHERE media_type='series'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    sqlx::query("UPDATE series_episodes SET has_file=0")
        .execute(&mut *tx)
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
        let media_file_id = upsert_media_file(&mut tx, "series", item.id, file, "scan").await?;
        let last = last.unwrap_or(first).max(first);
        let mut linked = false;
        for episode in first..=last {
            if let Some(ep_id)=sqlx::query_scalar::<_,i64>(
                "SELECT id FROM series_episodes WHERE series_id=? AND season_number=? AND episode_number=?"
            ).bind(item.id).bind(season).bind(episode).fetch_optional(&mut *tx).await.map_err(internal)? {
                sqlx::query("UPDATE series_episodes SET has_file=1,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                    .bind(ep_id).execute(&mut *tx).await.map_err(internal)?;
                sqlx::query(r#"INSERT INTO episode_files(episode_id,media_file_id) VALUES(?,?)
                    ON CONFLICT(episode_id,media_file_id) DO NOTHING"#)
                    .bind(ep_id).bind(media_file_id).execute(&mut *tx).await.map_err(internal)?;
                linked=true;
            }
        }
        if linked {
            matched += 1;
        }
    }
    tx.commit().await.map_err(internal)?;
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
    tx: &mut sqlx::SqliteConnection,
    media_type: &str,
    media_id: i64,
    path: &Path,
    discovered_by: &str,
) -> Result<i64, (StatusCode, String)> {
    let name = path.file_name().and_then(|x| x.to_str()).unwrap_or("");
    let parsed = releases::parse(name);
    let size = fs::metadata(path).ok().map(|m| m.len() as i64);
    let quality_json = serde_json::to_string(&parsed).unwrap_or_else(|_| "{}".into());
    // A rescan only has the bare filename to go on, which is strictly weaker
    // than the metadata an actual import already verified from the release
    // title (resolution/source/codec/HDR/audio/language, and the stable
    // quality_score computed from it). Clobbering that on every rescan was
    // losing real information — a 2160p/HDR/Spanish file would come back as
    // NULLs and score=0 just because its filename alone doesn't spell that
    // out, which then fed wrong upgrade/cutoff decisions. Only a row that
    // was itself never import-verified (discovered_by != 'import') gets its
    // quality fields refreshed from this scan's reparse.
    sqlx::query_scalar::<_,i64>(r#"
        INSERT INTO media_files(
          media_type,media_id,download_job_id,path,size_bytes,quality_json,source_release,
          resolution,source,codec,hdr,audio,language,quality_score,file_exists,discovered_by,verified_at
        ) VALUES(?,?,NULL,?,?,?,?,?,?,?,?,?,?,?,1,?,CURRENT_TIMESTAMP)
        ON CONFLICT(path) DO UPDATE SET
          media_type=excluded.media_type,media_id=excluded.media_id,size_bytes=excluded.size_bytes,
          file_exists=1,verified_at=CURRENT_TIMESTAMP,
          quality_json=CASE WHEN media_files.discovered_by='import' THEN media_files.quality_json ELSE excluded.quality_json END,
          source_release=CASE WHEN media_files.discovered_by='import' THEN media_files.source_release ELSE excluded.source_release END,
          resolution=CASE WHEN media_files.discovered_by='import' THEN media_files.resolution ELSE excluded.resolution END,
          source=CASE WHEN media_files.discovered_by='import' THEN media_files.source ELSE excluded.source END,
          codec=CASE WHEN media_files.discovered_by='import' THEN media_files.codec ELSE excluded.codec END,
          hdr=CASE WHEN media_files.discovered_by='import' THEN media_files.hdr ELSE excluded.hdr END,
          audio=CASE WHEN media_files.discovered_by='import' THEN media_files.audio ELSE excluded.audio END,
          language=CASE WHEN media_files.discovered_by='import' THEN media_files.language ELSE excluded.language END,
          quality_score=CASE WHEN media_files.discovered_by='import' THEN media_files.quality_score ELSE excluded.quality_score END
        RETURNING id
    "#)
        .bind(media_type).bind(media_id).bind(path.to_string_lossy().to_string()).bind(size)
        .bind(&quality_json).bind(name)
        .bind(parsed.resolution).bind(parsed.source).bind(parsed.codec).bind(parsed.hdr)
        .bind(parsed.audio).bind(parsed.language).bind(parsed.score).bind(discovered_by)
        .fetch_one(&mut *tx).await.map_err(internal)
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
        // file_type() reflects the directory entry itself, not what it
        // points to, so a symlink is never silently treated as a plain
        // directory here — Path::is_dir() would follow it. Not recursing
        // into one at all is what keeps a link back to an ancestor (or a
        // cycle between two directories, both easy to create on a NAS) from
        // recursing until the stack or the process's open-file limit gives
        // out and takes the scan down with it.
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let p = entry.path();
        if file_type.is_dir() {
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
    // `\b` after the optional second episode number matters: without it, a
    // release like "S01E02.1080p.mkv" let the digit run in "1080p" get
    // captured as a bogus second episode ("2 to 108"), which would then mark
    // over a hundred episodes as having a file. See importer/naming.rs's
    // parse_episode_numbers, which already carries this fix.
    let patterns = [
        r"(?i)S(\d{1,2})E(\d{1,3})(?:[-_. ]?E?(\d{1,3})\b)?",
        r"(?i)(\d{1,2})x(\d{1,3})(?:[-_. ]?(\d{1,3})\b)?",
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
