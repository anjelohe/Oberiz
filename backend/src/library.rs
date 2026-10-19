use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use crate::{AppState, history, library_roots, qbittorrent, releases, settings};

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
    pub error: Option<String>,
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
    original: Option<String>,
}
#[derive(Clone)]
struct SeriesCandidate {
    id: i64,
    name: String,
    original: Option<String>,
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
        out.extend(scan_media(&state, "movie").await?);
    }
    if target == "all" || target == "series" {
        out.extend(scan_media(&state, "series").await?);
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
    pub available_episodes: i64,
    pub missing_aired_episodes: i64,
    pub future_episodes: i64,
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
    let (available_episodes,missing_aired_episodes,future_episodes)=sqlx::query_as::<_,(i64,i64,i64)>(
        "SELECT COALESCE(SUM(has_file=1),0),COALESCE(SUM(has_file=0 AND (air_date IS NULL OR air_date<=date('now','localtime'))),0),COALESCE(SUM(has_file=0 AND air_date>date('now','localtime')),0) FROM series_episodes WHERE season_number>0"
    ).fetch_one(&state.db).await.map_err(internal)?;
    Ok(Json(LibrarySummary {
        movie_files,
        series_files,
        movie_bytes,
        series_bytes,
        total_bytes: movie_bytes + series_bytes,
        available_episodes,
        missing_aired_episodes,
        future_episodes,
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

async fn scan_movies_root(
    state: &AppState,
    root: String,
) -> Result<RescanResult, (StatusCode, String)> {
    let root_path = PathBuf::from(&root);
    if !root_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("La ruta Movies no existe: {}", root_path.display()),
        ));
    }

    let scan_id = start_scan(state, "movie", &root).await?;
    let files = tokio::task::spawn_blocking(move || collect_video_files(&root_path))
        .await
        .map_err(internal)?
        .map_err(fs_error)?;
    // A mount point that exists but is empty (a network share that hasn't
    // finished mounting yet, a NAS blip) would otherwise look identical to a
    // library with nothing left in it — and the unconditional file_exists=0
    // reset below would then tell automation every movie is missing and
    // re-download the entire library. Refuse instead of guessing.
    if files.is_empty() && root_has_registered_files(state, "movie", &root).await? {
        let message = "No video files found under the configured Movies path. Refusing to mark the library as missing — check that the path is mounted correctly.".to_string();
        finish_scan(state, scan_id, 0, 0, Some(&message)).await?;
        return Err((StatusCode::CONFLICT, message));
    }
    let movies = sqlx::query_as::<_, (i64, String, Option<i32>, Option<String>)>(
        "SELECT id,title,year,original_title FROM movies",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?
    .into_iter()
    .map(|x| MovieCandidate {
        id: x.0,
        title: x.1,
        year: x.2,
        original: x.3,
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
    invalidate_root_files(&mut tx, "movie", &root).await?;

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
            let title = best_alias(&haystack, &movie.title, movie.original.as_deref());
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
                        best_alias(&haystack, &current.title, current.original.as_deref()).len()
                            < title.len()
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
        error: None,
    })
}

async fn scan_series_root(
    state: &AppState,
    root: String,
) -> Result<RescanResult, (StatusCode, String)> {
    let root_path = PathBuf::from(&root);
    if !root_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("La ruta Series no existe: {}", root_path.display()),
        ));
    }

    let scan_id = start_scan(state, "series", &root).await?;
    let files = tokio::task::spawn_blocking(move || collect_video_files(&root_path))
        .await
        .map_err(internal)?
        .map_err(fs_error)?;
    // See scan_movies: an empty-but-present mount point must not be read as
    // "the whole library is gone", or the unconditional resets below would
    // make automation re-download every monitored series from scratch.
    if files.is_empty() && root_has_registered_files(state, "series", &root).await? {
        let message = "No video files found under the configured Series path. Refusing to mark the library as missing — check that the path is mounted correctly.".to_string();
        finish_scan(state, scan_id, 0, 0, Some(&message)).await?;
        return Err((StatusCode::CONFLICT, message));
    }
    let series = sqlx::query_as::<_, (i64, String, Option<String>)>(
        "SELECT id,name,original_name FROM series",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?
    .into_iter()
    .map(|x| SeriesCandidate {
        id: x.0,
        name: x.1,
        original: x.2,
    })
    .collect::<Vec<_>>();

    let known_paths: HashMap<String, i64> = sqlx::query_as::<_, (String, i64)>(
        "SELECT path,media_id FROM media_files WHERE media_type='series'",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?
    .into_iter()
    .collect();
    // See scan_movies for why this is one transaction: without it, other
    // connections could observe the library mid-reset (everything marked
    // missing) while this loop is still rebuilding it, and a crash midway
    // would leave that half-reset state persisted instead of rolling back.
    let mut tx = state.db.begin().await.map_err(internal)?;
    invalidate_root_files(&mut tx, "series", &root).await?;

    let mut matched = 0usize;
    for file in &files {
        let Some((season, first, last)) =
            parse_episode_span(file.file_name().and_then(|x| x.to_str()).unwrap_or(""))
        else {
            continue;
        };
        let haystack = normalized(&file.to_string_lossy());
        let known = known_paths
            .get(&file.to_string_lossy().to_string())
            .and_then(|id| series.iter().find(|s| s.id == *id));
        let mut best: Option<&SeriesCandidate> = known;
        for item in series.iter().filter(|_| known.is_none()) {
            let name = best_alias(&haystack, &item.name, item.original.as_deref());
            if name.len() < 2 || !haystack.contains(&name) {
                continue;
            }
            if best
                .as_ref()
                .map(|b| best_alias(&haystack, &b.name, b.original.as_deref()).len())
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
    sqlx::query("UPDATE series_episodes SET has_file=EXISTS(SELECT 1 FROM episode_files ef JOIN media_files mf ON mf.id=ef.media_file_id WHERE ef.episode_id=series_episodes.id AND mf.file_exists=1),updated_at=CURRENT_TIMESTAMP").execute(&mut *tx).await.map_err(internal)?;
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
        error: None,
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
    // External files and imported files must share the profile's stable score scale.
    // Using ParsedRelease::score here can make a discovered copy appear better than
    // any profile-evaluated upgrade, simply because the two scales differ.
    let mut quality_score = parsed.score;
    if media_type == "movie" {
        type ScanMovieIdentity = (String, Option<String>, Option<i32>, Option<i64>);
        let identity: Option<ScanMovieIdentity> = sqlx::query_as(
            "SELECT title,original_title,year,quality_profile_id FROM movies WHERE id=?",
        )
        .bind(media_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal)?;
        if let Some((title, original, year, Some(profile_id))) = identity {
            let profile = crate::profiles::get_quality_profile_by_id(&mut *tx, profile_id).await?;
            let language = crate::profiles::language_for_profile(&mut *tx, &profile).await?;
            let candidate = crate::cardigann::ReleaseResult {
                indexer_id: String::new(),
                indexer_name: String::new(),
                title: name.into(),
                details_url: None,
                download_url: None,
                size_bytes: size,
                seeders: None,
                leechers: None,
                category: None,
                published: None,
                score: parsed.score,
                resolution: parsed.resolution.clone(),
                source: parsed.source.clone(),
                codec: parsed.codec.clone(),
                hdr: parsed.hdr.clone(),
                audio: parsed.audio.clone(),
                language: parsed.language.clone(),
                base_score: parsed.score,
                profile_score: 0,
                match_score: 0,
                accepted: false,
                rejection_id: None,
                reasons: vec![],
                rejection_reasons: vec![],
            };
            let evaluation = crate::profiles::evaluate_release(
                &candidate,
                &profile,
                language.as_ref(),
                &title,
                original.as_deref(),
                year,
                media_type,
                None,
            );
            quality_score = evaluation.match_score + evaluation.profile_score;
        }
    }
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
        .bind(parsed.audio).bind(parsed.language).bind(quality_score).bind(discovered_by)
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
    collect_recursive(root, &mut out, &mut HashSet::new())?;
    let mut seen = HashSet::new();
    out.retain(|path| seen.insert(fs::canonicalize(path).unwrap_or_else(|_| path.clone())));
    Ok(out)
}
fn collect_recursive(
    path: &Path,
    out: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) -> std::io::Result<()> {
    if path.is_file() {
        if is_video(path) {
            out.push(path.to_path_buf());
        }
        return Ok(());
    }
    // Also covers Windows junctions/reparse points that are not reported as
    // symlinks: do not revisit a canonical directory through another alias.
    if !visited.insert(fs::canonicalize(path)?) {
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
            collect_recursive(&p, out, visited)?;
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
pub(crate) fn normalized(value: &str) -> String {
    value
        .to_lowercase()
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
pub(crate) fn parse_episode_span(name: &str) -> Option<(i32, i32, Option<i32>)> {
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
static SCAN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn best_alias(haystack: &str, title: &str, original: Option<&str>) -> String {
    [Some(title), original]
        .into_iter()
        .flatten()
        .map(normalized)
        .filter(|name| name.len() >= 2 && haystack.contains(name))
        .max_by_key(String::len)
        .unwrap_or_default()
}

async fn root_has_registered_files(
    state: &AppState,
    kind: &str,
    root: &str,
) -> Result<bool, (StatusCode, String)> {
    let paths = sqlx::query_scalar::<_, String>(
        "SELECT path FROM media_files WHERE media_type=? AND file_exists=1",
    )
    .bind(kind)
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;
    Ok(paths
        .iter()
        .any(|path| library_roots::contains_path(root, path)))
}

async fn invalidate_root_files(
    tx: &mut sqlx::SqliteConnection,
    kind: &str,
    root: &str,
) -> Result<(), (StatusCode, String)> {
    let paths = sqlx::query_as::<_, (i64, String)>(
        "SELECT id,path FROM media_files WHERE media_type=? AND file_exists=1",
    )
    .bind(kind)
    .fetch_all(&mut *tx)
    .await
    .map_err(internal)?;
    for (id, path) in paths {
        if library_roots::contains_path(root, &path) {
            sqlx::query("UPDATE media_files SET file_exists=0 WHERE id=?")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(internal)?;
        }
    }
    Ok(())
}

fn independent_roots(roots: Vec<String>) -> Vec<String> {
    let mut roots: Vec<(String, String)> = roots
        .into_iter()
        .map(|root| {
            let actual = fs::canonicalize(&root)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| root.clone());
            (root, library_roots::path_key(&actual))
        })
        .collect();
    roots.sort_by_key(|(_, key)| key.len());
    let mut chosen: Vec<(String, String)> = Vec::new();
    for root in roots {
        if !chosen
            .iter()
            .any(|(_, key)| library_roots::contains_path(key, &root.1))
        {
            chosen.push(root);
        }
    }
    chosen.into_iter().map(|(path, _)| path).collect()
}

pub(crate) async fn scan_media(
    state: &AppState,
    kind: &str,
) -> Result<Vec<RescanResult>, (StatusCode, String)> {
    let _media_guard = crate::importer::MEDIA_OPERATION_GATE.lock().await;
    crate::importer::require_managed_files(state).await?;
    scan_media_managed(state, kind).await
}

async fn scan_media_managed(
    state: &AppState,
    kind: &str,
) -> Result<Vec<RescanResult>, (StatusCode, String)> {
    let _guard = SCAN_LOCK.lock().await;
    let roots = independent_roots(
        library_roots::roots(&state.db, kind)
            .await
            .map_err(internal)?,
    );
    if roots.is_empty() {
        return Ok(Vec::new());
    }
    let mut results = Vec::new();
    for root in roots {
        let result = if kind == "movie" {
            scan_movies_root(state, root.clone()).await
        } else {
            scan_series_root(state, root.clone()).await
        };
        match result {
            Ok(result) => results.push(result),
            Err((_, error)) => {
                // No reset has committed on a failed/empty/unmounted root.
                let id = start_scan(state, kind, &root).await?;
                finish_scan(state, id, 0, 0, Some(&error)).await?;
                results.push(RescanResult {
                    media_type: kind.into(),
                    root_path: root,
                    scanned_files: 0,
                    matched_files: 0,
                    unmatched_files: 0,
                    error: Some(error),
                });
            }
        }
    }
    if results.iter().all(|r| r.error.is_none()) {
        let table = if kind == "movie" { "movies" } else { "series" };
        sqlx::query(&format!("INSERT INTO library_discovery_checks(media_type,media_id,checked_at) SELECT ?,id,CURRENT_TIMESTAMP FROM {table} WHERE 1 ON CONFLICT(media_type,media_id) DO UPDATE SET checked_at=CURRENT_TIMESTAMP"))
            .bind(kind).execute(&state.db).await.map_err(internal)?;
    }
    Ok(results)
}

#[derive(Debug, Serialize)]
pub struct RootStatus {
    media_type: String,
    path: String,
    accessible: bool,
    last_scan_at: Option<String>,
    error: Option<String>,
}
pub async fn root_status(
    State(state): State<AppState>,
) -> Result<Json<Vec<RootStatus>>, (StatusCode, String)> {
    let _media_guard = crate::importer::MEDIA_OPERATION_GATE.lock().await;
    if crate::importer::external_mode(&state).await? {
        return Ok(Json(Vec::new()));
    }
    let mut rows = Vec::new();
    for kind in ["movie", "series"] {
        let scans=sqlx::query_as::<_,(String,Option<String>,Option<String>)>("SELECT root_path,completed_at,error FROM library_scans WHERE media_type=? AND id IN (SELECT MAX(id) FROM library_scans WHERE media_type=? GROUP BY root_path) ORDER BY id DESC")
            .bind(kind).bind(kind).fetch_all(&state.db).await.map_err(internal)?;
        for path in library_roots::roots(&state.db, kind)
            .await
            .map_err(internal)?
        {
            let latest = scans
                .iter()
                .find(|(root, _, _)| library_roots::contains_path(root, &path))
                .map(|(_, date, error)| (date.clone(), error.clone()));
            let accessible = Path::new(&path).is_dir();
            let (last_scan_at, error) = latest.unwrap_or_default();
            rows.push(RootStatus {
                media_type: kind.into(),
                path,
                accessible,
                last_scan_at,
                error,
            });
        }
    }
    Ok(Json(rows))
}

pub(crate) async fn ensure_discovered(
    state: &AppState,
    kind: &str,
    id: i64,
) -> Result<(), (StatusCode, String)> {
    let _media_guard = crate::importer::MEDIA_OPERATION_GATE.lock().await;
    if crate::importer::external_mode(state).await? {
        if kind == "series" {
            crate::external_library::reconcile_cached_series_locked(state, id).await?;
        }
        if settings::get_value(&state.db, "library.pause_on_stale_inventory")
            .await
            .map_err(internal)?
            .as_deref()
            != Some("true")
        {
            return Ok(());
        }
        let fresh: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM settings WHERE key='library.last_external_report' AND julianday(value)>=julianday('now','-24 hours'))")
            .fetch_one(&state.db).await.map_err(internal)?;
        if !fresh {
            return Err((StatusCode::CONFLICT,"Cinetta inventory missing or older than 24 hours. Acquisition is held to avoid duplicate downloads.".into()));
        }
        return Ok(());
    }
    let roots = independent_roots(
        library_roots::roots(&state.db, kind)
            .await
            .map_err(internal)?,
    );
    for root in &roots {
        if !Path::new(root).is_dir() {
            return Err((
                StatusCode::CONFLICT,
                format!(
                    "Library folder unavailable: {root}. Automatic acquisition is held to avoid duplicates."
                ),
            ));
        }
        let error=sqlx::query_scalar::<_,Option<String>>("SELECT error FROM library_scans WHERE media_type=? AND root_path=? ORDER BY id DESC LIMIT 1")
            .bind(kind).bind(root).fetch_optional(&state.db).await.map_err(internal)?.flatten();
        if let Some(error) = error {
            return Err((StatusCode::CONFLICT, error));
        }
    }
    let checked = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM library_discovery_checks WHERE media_type=? AND media_id=?",
    )
    .bind(kind)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?
        > 0;
    if !checked && !roots.is_empty() {
        let results = scan_media_managed(state, kind).await?;
        if let Some(error) = results.into_iter().find_map(|r| r.error) {
            return Err((StatusCode::CONFLICT, error));
        }
    }
    // A torrent downloaded elsewhere is not an Oberiz import job. Read its content
    // path to discover accessible completed files without changing qBittorrent.
    {
        let identity = media_identity(state, kind, id).await?;
        if let Ok(torrents) = qbittorrent::list_torrents_internal(state).await {
            for torrent in torrents.into_iter().filter(|t| {
                t.progress >= 0.999999
                    && torrent_matches(
                        &t.name,
                        &identity.0,
                        identity.1.as_deref(),
                        if kind == "movie" { identity.2 } else { None },
                    )
            }) {
                let path =
                    crate::importer::translate_remote_path(state, &torrent.content_path).await;
                if path.is_empty() || !Path::new(&path).exists() {
                    continue;
                }
                // Each completed torrent is considered, even if other episodes
                // of the same series were already indexed on another disk.
                let recent=sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM library_scans WHERE media_type=? AND root_path=? AND error IS NULL AND completed_at>=datetime('now','-120 minutes') AND (?='movie' OR completed_at>=COALESCE((SELECT metadata_synced_at FROM series WHERE id=?),'1970-01-01'))")
                    .bind(kind).bind(&path).bind(kind).bind(id).fetch_one(&state.db).await.map_err(internal)?;
                if recent > 0 {
                    continue;
                }
                let _guard = SCAN_LOCK.lock().await;
                if kind == "movie" {
                    scan_movies_root(state, path).await?;
                } else {
                    scan_series_root(state, path).await?;
                }
            }
        }
    }
    sqlx::query("INSERT INTO library_discovery_checks(media_type,media_id) VALUES(?,?) ON CONFLICT(media_type,media_id) DO UPDATE SET checked_at=CURRENT_TIMESTAMP")
        .bind(kind).bind(id).execute(&state.db).await.map_err(internal)?;
    Ok(())
}

async fn media_identity(
    state: &AppState,
    kind: &str,
    id: i64,
) -> Result<(String, Option<String>, Option<i32>), (StatusCode, String)> {
    let query = if kind == "movie" {
        "SELECT title,original_title,year FROM movies WHERE id=?"
    } else {
        "SELECT name,original_name,year FROM series WHERE id=?"
    };
    sqlx::query_as(query)
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)
}

fn torrent_matches(name: &str, title: &str, original: Option<&str>, year: Option<i32>) -> bool {
    let haystack = normalized(name);
    if best_alias(&haystack, title, original).is_empty() {
        return false;
    }
    // A different explicit release year must not make an unrelated remake look present.
    if let Some(year) = year {
        let years = regex::Regex::new(r"\b(?:19|20)\d{2}\b").unwrap();
        if years.is_match(&haystack) && !haystack.contains(&year.to_string()) {
            return false;
        }
    }
    true
}

fn torrent_covers(name: &str, season: Option<i32>, episode: Option<i32>) -> bool {
    let Some(season) = season else {
        return true;
    };
    if let Some((s, first, last)) = parse_episode_span(name) {
        return s == season && episode.is_some_and(|ep| ep >= first && ep <= last.unwrap_or(first));
    }
    let pack = regex::Regex::new(r"(?i)\b(?:S|SEASON[ ._-]*|TEMPORADA[ ._-]*)(\d{1,2})\b").unwrap();
    pack.captures_iter(name)
        .any(|c| c[1].parse::<i32>().ok() == Some(season))
        || regex::Regex::new(r"(?i)\b(?:COMPLETE SERIES|SERIE COMPLETA|THE COMPLETE SERIES)\b")
            .unwrap()
            .is_match(name)
}

pub(crate) async fn existing_download_covers(
    state: &AppState,
    kind: &str,
    id: i64,
    season: Option<i32>,
    episode: Option<i32>,
    has_file: bool,
) -> Result<bool, (StatusCode, String)> {
    let identity = media_identity(state, kind, id).await?;
    let torrents = qbittorrent::list_torrents_internal(state).await?;
    Ok(torrents.iter().any(|torrent| {
        (!has_file || torrent.progress < 0.999999)
            && torrent_matches(
                &torrent.name,
                &identity.0,
                identity.1.as_deref(),
                if kind == "movie" { identity.2 } else { None },
            )
            && torrent_covers(&torrent.name, season, episode)
    }))
}

pub fn spawn_scan_scheduler(state: AppState) {
    tokio::spawn(async move {
        let mut last = std::time::Instant::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let enabled = settings::get_value(&state.db, "library.scan_enabled")
                .await
                .ok()
                .flatten()
                .as_deref()
                == Some("true");
            let minutes = settings::get_value(&state.db, "library.scan_interval_minutes")
                .await
                .ok()
                .flatten()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(120)
                .clamp(5, 10080);
            if !enabled
                || crate::importer::external_mode(&state).await.unwrap_or(true)
                || last.elapsed().as_secs() < minutes * 60
            {
                continue;
            }
            last = std::time::Instant::now();
            for kind in ["movie", "series"] {
                if library_roots::roots(&state.db, kind)
                    .await
                    .is_ok_and(|r| !r.is_empty())
                    && let Err((_, error)) = scan_media(&state, kind).await
                {
                    history::record(&state.db, "library.scan.error", kind, Some(&error), "error")
                        .await;
                }
            }
        }
    });
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
fn fs_error(e: std::io::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct TestFolder(PathBuf);
    impl TestFolder {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "oberiz-library-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn folder(&self, name: &str) -> String {
            let path = self.0.join(name);
            fs::create_dir_all(&path).unwrap();
            path.to_string_lossy().into_owned()
        }
        fn video(&self, root: &str, name: &str) -> String {
            let path = Path::new(root).join(name);
            fs::write(&path, b"test-video").unwrap();
            path.to_string_lossy().into_owned()
        }
    }
    impl Drop for TestFolder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    async fn state() -> AppState {
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        AppState {
            db,
            http: reqwest::Client::new(),
            automation_runtime: crate::automation::AutomationRuntime::default(),
            download_client: Arc::new(crate::download_client::QBittorrentClient),
            torrent_list_cache: Arc::new(Mutex::new(Some(crate::TorrentListCache {
                fetched_at: std::time::Instant::now(),
                torrents: Vec::new(),
            }))),
            grab_lock: Arc::new(Mutex::new(())),
        }
    }
    #[tokio::test]
    async fn stale_cinetta_pause_is_opt_in_and_can_be_disabled() {
        let state = state().await;
        settings::set_value(&state.db, "library.source", "cinetta")
            .await
            .unwrap();
        assert!(
            !crate::settings::get_settings(axum::extract::State(state.clone()))
                .await
                .unwrap()
                .0
                .pause_on_stale_inventory
        );
        ensure_discovered(&state, "movie", 1).await.unwrap();
        let set_pause = |enabled| crate::settings::UpdateSettingsRequest {
            pause_on_stale_inventory: Some(enabled),
            ..Default::default()
        };
        crate::settings::update_settings(
            axum::extract::State(state.clone()),
            Json(set_pause(true)),
        )
        .await
        .unwrap();
        assert_eq!(
            ensure_discovered(&state, "movie", 1).await.unwrap_err().0,
            StatusCode::CONFLICT
        );
        settings::set_value(
            &state.db,
            "library.last_external_report",
            "2000-01-01T00:00:00Z",
        )
        .await
        .unwrap();
        assert!(ensure_discovered(&state, "movie", 1).await.is_err());
        sqlx::query("UPDATE settings SET value=strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE key='library.last_external_report'").execute(&state.db).await.unwrap();
        ensure_discovered(&state, "movie", 1).await.unwrap();
        settings::set_value(
            &state.db,
            "library.last_external_report",
            "2000-01-01T00:00:00Z",
        )
        .await
        .unwrap();
        crate::settings::update_settings(
            axum::extract::State(state.clone()),
            Json(set_pause(false)),
        )
        .await
        .unwrap();
        ensure_discovered(&state, "movie", 1).await.unwrap();
    }
    async fn roots(state: &AppState, kind: &str, paths: Vec<String>) {
        settings::set_value(
            &state.db,
            if kind == "movie" {
                "library.movie_roots"
            } else {
                "library.series_roots"
            },
            &serde_json::to_string(&paths).unwrap(),
        )
        .await
        .unwrap();
    }
    async fn show(state: &AppState) {
        sqlx::query("INSERT INTO series(id,tmdb_id,name,original_name,monitor_mode) VALUES(1,1,'Nombre español','Original Show','future')").execute(&state.db).await.unwrap();
        sqlx::query("INSERT INTO series_seasons(id,series_id,season_number,name,episode_count) VALUES(1,1,1,'Season 1',3)").execute(&state.db).await.unwrap();
        for ep in 1..=3 {
            sqlx::query("INSERT INTO series_episodes(series_id,season_id,season_number,episode_number,name,air_date) VALUES(1,1,1,?,'Episode',date('now','localtime',?))")
                .bind(ep).bind(format!("{} days",ep-2)).execute(&state.db).await.unwrap();
        }
    }

    #[test]
    fn torrent_coverage_is_episode_and_season_specific() {
        assert!(torrent_covers(
            "Original.Show.S01E02-E04.1080p",
            Some(1),
            Some(3)
        ));
        assert!(!torrent_covers(
            "Original.Show.S01E02.1080p",
            Some(1),
            Some(3)
        ));
        assert!(!torrent_covers(
            "Original.Show.S01E02.1080p",
            Some(2),
            Some(2)
        ));
        assert!(torrent_covers("Original.Show.S01.PACK", Some(1), Some(9)));
        assert!(!torrent_covers("Original.Show.S02.PACK", Some(1), Some(9)));
        assert!(torrent_covers(
            "Original Show Serie completa",
            Some(2),
            Some(3)
        ));
        assert!(!torrent_matches(
            "The Thing 2011",
            "The Thing",
            None,
            Some(1982)
        ));
        assert!(torrent_matches(
            "Original.Show.S01E02",
            "Nombre español",
            Some("Original Show"),
            None
        ));
    }

    #[tokio::test]
    async fn multiple_disks_overlap_and_disconnection_preserve_files() {
        let state = state().await;
        let folder = TestFolder::new();
        let a = folder.folder("a");
        let nested = folder.folder("a/nested");
        let b = folder.folder("b");
        folder.video(&nested, "Alpha 2025.mkv");
        folder.video(&b, "Beta 2025.mkv");
        sqlx::query(
            "INSERT INTO movies(tmdb_id,title,year) VALUES(1,'Alpha',2025),(2,'Beta',2025)",
        )
        .execute(&state.db)
        .await
        .unwrap();
        roots(&state, "movie", vec![a.clone(), nested, b.clone(), a]).await;
        let scans = scan_media(&state, "movie").await.unwrap();
        assert_eq!(scans.len(), 2);
        assert_eq!(scans.iter().map(|s| s.matched_files).sum::<usize>(), 2);
        let totals = summary(State(state.clone())).await.unwrap().0;
        assert_eq!(totals.movie_files, 2);
        assert_eq!(totals.total_bytes, 20);
        fs::rename(&b, folder.0.join("unmounted-b")).unwrap();
        let scans = scan_media(&state, "movie").await.unwrap();
        assert!(scans.iter().any(|r| r.error.is_some()));
        assert_eq!(
            summary(State(state.clone())).await.unwrap().0.movie_files,
            2
        );
        assert_eq!(
            ensure_discovered(&state, "movie", 2).await.unwrap_err().0,
            StatusCode::CONFLICT
        );
        let status = root_status(State(state.clone())).await.unwrap().0;
        assert!(
            status
                .iter()
                .any(|r| r.path == b && !r.accessible && r.error.is_some())
        );
        assert!(
            status
                .iter()
                .filter(|r| r.accessible)
                .all(|r| r.last_scan_at.is_some())
        );
        roots(&state, "movie", Vec::new()).await;
        assert!(scan_media(&state, "movie").await.unwrap().is_empty());
        assert_eq!(summary(State(state)).await.unwrap().0.movie_files, 2);
    }

    #[tokio::test]
    async fn empty_mount_does_not_clear_series_availability() {
        let state = state().await;
        show(&state).await;
        let folder = TestFolder::new();
        let root = folder.folder("shows");
        let file = folder.video(&root, "Original.Show.S01E01.1080p.mkv");
        roots(&state, "series", vec![root]).await;
        assert_eq!(
            scan_media(&state, "series").await.unwrap()[0].matched_files,
            1
        );
        fs::rename(file, folder.0.join("offline-video")).unwrap();
        assert!(
            scan_media(&state, "series").await.unwrap()[0]
                .error
                .is_some()
        );
        assert_eq!(summary(State(state)).await.unwrap().0.available_episodes, 1);
    }

    #[tokio::test]
    async fn completed_external_episode_is_found_even_with_other_files_indexed() {
        let state = state().await;
        show(&state).await;
        let folder = TestFolder::new();
        let a = folder.folder("first");
        let b = folder.folder("external");
        folder.video(&a, "Original.Show.S01E01.mkv");
        let external = folder.video(&b, "Original.Show.S01E02.mkv");
        roots(&state, "series", vec![a]).await;
        scan_media(&state, "series").await.unwrap();
        let torrent:qbittorrent::QBittorrentTorrent=serde_json::from_value(serde_json::json!({"hash":"test","name":"Original.Show.S01E02","content_path":external,"progress":1.0,"state":"pausedUP"})).unwrap();
        *state.torrent_list_cache.lock().await = Some(crate::TorrentListCache {
            fetched_at: std::time::Instant::now(),
            torrents: vec![torrent],
        });
        ensure_discovered(&state, "series", 1).await.unwrap();
        assert_eq!(
            summary(State(state.clone()))
                .await
                .unwrap()
                .0
                .available_episodes,
            2
        );
        assert!(
            !existing_download_covers(&state, "series", 1, Some(1), Some(2), true)
                .await
                .unwrap()
        );
        assert!(
            existing_download_covers(&state, "series", 1, Some(1), Some(2), false)
                .await
                .unwrap()
        );
        let cache = state.torrent_list_cache.lock().await;
        assert_eq!(cache.as_ref().unwrap().torrents[0].state, "pausedUP");
    }

    #[tokio::test]
    async fn invalid_library_configuration_does_not_partially_save_the_form() {
        let state = state().await;
        settings::set_value(&state.db, "paths.movies", "/original")
            .await
            .unwrap();
        let payload:settings::UpdateSettingsRequest=serde_json::from_value(serde_json::json!({"movies_path":"/changed","qbittorrent_path_mappings":[{"remote":"/remote","local":"relative-path"}]})).unwrap();
        assert_eq!(
            settings::update_settings(State(state.clone()), Json(payload))
                .await
                .unwrap_err(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            settings::get_value(&state.db, "paths.movies")
                .await
                .unwrap()
                .as_deref(),
            Some("/original")
        );
    }

    #[tokio::test]
    async fn future_tracking_keeps_recently_aired_episodes_and_manual_overrides() {
        let state = state().await;
        show(&state).await;
        // Tracking began yesterday: an episode remains wanted today and beyond.
        sqlx::query(
            "UPDATE series SET monitor_future_since=date('now','localtime','-1 day') WHERE id=1",
        )
        .execute(&state.db)
        .await
        .unwrap();
        crate::series::apply_monitor_mode_internal(&state, 1, "future")
            .await
            .unwrap();
        let monitored: Vec<i64> =
            sqlx::query_scalar("SELECT monitored FROM series_episodes ORDER BY episode_number")
                .fetch_all(&state.db)
                .await
                .unwrap();
        assert_eq!(monitored, vec![1, 1, 1]);
        sqlx::query(
            "UPDATE series_episodes SET monitor_override=0,monitored=0 WHERE episode_number=2",
        )
        .execute(&state.db)
        .await
        .unwrap();
        sqlx::query("UPDATE series_seasons SET monitored=1,monitor_override=1,follow_future_episodes=1,follow_future_since=date('now','localtime','-1 day') WHERE id=1").execute(&state.db).await.unwrap();
        crate::series::apply_monitor_mode_internal(&state, 1, "none")
            .await
            .unwrap();
        let monitored: Vec<i64> =
            sqlx::query_scalar("SELECT monitored FROM series_episodes ORDER BY episode_number")
                .fetch_all(&state.db)
                .await
                .unwrap();
        assert_eq!(monitored, vec![1, 0, 1]);
        assert_eq!(
            crate::series::availability_label(2, 1, 0, 3, true),
            "partially_available"
        );
        assert_eq!(
            crate::series::availability_label(3, 0, 0, 3, false),
            "up_to_date"
        );
        assert_eq!(
            crate::series::availability_label(3, 0, 0, 3, true),
            "complete"
        );
        assert_eq!(
            crate::series::availability_label(2, 0, 1, 3, true),
            "up_to_date"
        );
    }
}
