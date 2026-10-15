//! Library reported by an external media server (Cinetta) through the public
//! API. Oberiz never opens these paths: they only feed availability, upgrade
//! scoring and the list of older versions an Oberiz upgrade has superseded.
//!
//! Reported files live in `media_files` like scanned or imported ones, so every
//! existing availability, request-status and upgrade check works unchanged.
//! Their `path` carries the `cinetta:` prefix, which no local folder can
//! contain, so library scans (which invalidate files by root folder) never
//! touch them and the same file reported by both sources never collides.
use crate::{AppState, history, library_roots, profiles, public_api, releases, series};
use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{StatusCode, request::Parts},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

pub(crate) const PREFIX: &str = "cinetta:";
const MAX_ITEMS: usize = 1_000_000;
const MAX_ROOTS: usize = 1024;
static REPORT_SLOTS: LazyLock<Arc<tokio::sync::Semaphore>> =
    LazyLock::new(|| Arc::new(tokio::sync::Semaphore::new(2)));

pub struct AuthorizedInventory {
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl FromRequestParts<AppState> for AuthorizedInventory {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Error> {
        public_api::authorize(state, &parts.headers).await?;
        let permit = REPORT_SLOTS.clone().try_acquire_owned().map_err(|_| {
            (
                StatusCode::TOO_MANY_REQUESTS,
                "Inventory receiver busy; retry later".into(),
            )
        })?;
        Ok(Self { _permit: permit })
    }
}
type Error = (StatusCode, String);

#[derive(Debug, Deserialize)]
pub struct ReportedFile {
    pub path: String,
    pub size: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub video_codec: Option<String>,
    pub hdr: Option<String>,
    pub audio_codec: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReportedMovie {
    pub tmdb_id: i64,
    #[serde(flatten)]
    pub file: ReportedFile,
}

#[derive(Debug, Deserialize)]
pub struct ReportedEpisode {
    pub series_tmdb_id: i64,
    pub season: i32,
    pub episode: i32,
    /// Last episode of a multi-episode file (S01E01E02 → episode 1, end 2).
    /// The same file may also simply be reported once per episode.
    #[serde(default)]
    pub episode_end: Option<i32>,
    #[serde(flatten)]
    pub file: ReportedFile,
}

/// One of the client's library folders and whether it could read it now.
#[derive(Debug, Deserialize)]
pub struct ReportedRoot {
    pub path: String,
    pub available: bool,
}

#[derive(Debug, Deserialize)]
pub struct LibraryReport {
    #[serde(default)]
    pub movies: Vec<ReportedMovie>,
    #[serde(default)]
    pub episodes: Vec<ReportedEpisode>,
    /// Files previously reported under an unavailable root are kept as they
    /// were: an offline disk must never look like a deleted library.
    #[serde(default)]
    pub roots: Vec<ReportedRoot>,
    /// Explicitly confirms omissions under reported available, nonempty roots.
    /// Without confirmation reports are additive, even for a single omission.
    #[serde(default)]
    pub allow_large_removal: bool,
}

#[derive(Debug, Serialize)]
pub struct Superseded {
    pub media_type: String,
    pub tmdb_id: i64,
    pub season: Option<i32>,
    pub episode: Option<i32>,
    pub path: String,
    pub replacement_path: String,
}

#[derive(Debug, Serialize)]
pub struct LibraryReportResult {
    pub movie_files: usize,
    pub episode_files: usize,
    pub ignored_files: usize,
    pub missing_files: usize,
    /// Files kept as present because their root folder is offline.
    pub kept_offline_files: usize,
    pub superseded: Vec<Superseded>,
}

struct Prepared {
    media_type: &'static str,
    media_id: i64,
    /// Every episode this file contains; empty for a movie.
    episode_ids: Vec<i64>,
    path: String,
    size: Option<i64>,
    parsed: releases::ParsedRelease,
    score: i32,
}

static LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn report(
    State(state): State<AppState>,
    _authorization: AuthorizedInventory,
    Json(report): Json<LibraryReport>,
) -> Result<Json<LibraryReportResult>, Error> {
    validate(&report)?;
    let _guard = LOCK.lock().await;
    let _media_guard = crate::importer::MEDIA_OPERATION_GATE.lock().await;
    if !crate::importer::external_mode(&state).await? {
        return Err((
            StatusCode::CONFLICT,
            "Select Cinetta as the library source before reporting inventory".into(),
        ));
    }
    Ok(Json(apply(&state, report).await?))
}

fn validate(report: &LibraryReport) -> Result<(), Error> {
    if report.movies.len() + report.episodes.len() > MAX_ITEMS || report.roots.len() > MAX_ROOTS {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "Too many files".into()));
    }
    for path in report
        .movies
        .iter()
        .map(|m| &m.file.path)
        .chain(report.episodes.iter().map(|e| &e.file.path))
        .chain(report.roots.iter().map(|r| &r.path))
    {
        if path.trim().is_empty() || path.len() > 4096 || path.contains('\0') {
            return Err((StatusCode::BAD_REQUEST, "Invalid file path".into()));
        }
    }
    if report.episodes.iter().any(|e| {
        e.series_tmdb_id <= 0
            || e.season < 0
            || e.episode <= 0
            || e.episode_end.is_some_and(|end| {
                end.checked_sub(e.episode)
                    .is_none_or(|span| !(0..=50).contains(&span))
            })
    }) {
        return Err((StatusCode::BAD_REQUEST, "Invalid episode range".into()));
    }
    let mut identities = HashMap::new();
    for (path, identity) in report
        .movies
        .iter()
        .map(|m| (&m.file.path, ("movie", m.tmdb_id)))
        .chain(
            report
                .episodes
                .iter()
                .map(|e| (&e.file.path, ("series", e.series_tmdb_id))),
        )
    {
        if identity.1 <= 0
            || identities
                .insert(path, identity)
                .is_some_and(|old| old != identity)
        {
            return Err((
                StatusCode::BAD_REQUEST,
                "Conflicting file identities".into(),
            ));
        }
    }
    for file in report
        .movies
        .iter()
        .map(|m| &m.file)
        .chain(report.episodes.iter().map(|e| &e.file))
    {
        if file.size.is_some_and(|n| n < 0)
            || [file.width, file.height]
                .into_iter()
                .flatten()
                .any(|n| !(1..=32768).contains(&n))
            || [&file.video_codec, &file.hdr, &file.audio_codec]
                .into_iter()
                .flatten()
                .any(|s| s.len() > 128 || s.contains('\0'))
        {
            return Err((StatusCode::BAD_REQUEST, "Invalid file metadata".into()));
        }
    }
    Ok(())
}

async fn apply(state: &AppState, report: LibraryReport) -> Result<LibraryReportResult, Error> {
    validate(&report)?;
    let external = crate::importer::external_mode(state).await?;
    let mappings = library_roots::mappings(&state.db).await.map_err(internal)?;
    type MovieRow = (i64, i64, String, Option<String>, Option<i32>, Option<i64>);
    let movies: HashMap<i64, MovieRow> = sqlx::query_as::<_, MovieRow>(
        "SELECT id,tmdb_id,title,original_title,year,quality_profile_id FROM movies",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?
    .into_iter()
    .map(|row| (row.1, row))
    .collect();
    let series_rows: HashMap<i64, (i64, String, Option<String>, Option<i32>)> =
        sqlx::query_as::<_, (i64, i64, String, Option<String>, Option<i32>)>(
            "SELECT id,tmdb_id,name,original_name,year FROM series",
        )
        .fetch_all(&state.db)
        .await
        .map_err(internal)?
        .into_iter()
        .map(|(id, tmdb, name, original, year)| (tmdb, (id, name, original, year)))
        .collect();
    let episode_ids: HashMap<(i64, i32, i32), i64> = sqlx::query_as::<_, (i64, i64, i32, i32)>(
        "SELECT id,series_id,season_number,episode_number FROM series_episodes",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?
    .into_iter()
    .map(|(id, series, season, episode)| ((series, season, episode), id))
    .collect();

    // Scores are computed before the write transaction: they read profiles
    // through the pool, and doing it inside would hold the write lock for the
    // whole (potentially long) evaluation.
    let mut prepared = Vec::new();
    let mut ignored = 0usize;
    let mut profile_cache = ProfileCache::default();
    for movie in &report.movies {
        let Some((id, _, title, original, year, profile_id)) = movies.get(&movie.tmdb_id) else {
            ignored += 1;
            continue;
        };
        let parsed = releases::parse(&descriptor(&movie.file));
        let score = profile_cache
            .score(
                state,
                *profile_id,
                &parsed,
                title,
                original.as_deref(),
                *year,
                "movie",
                None,
            )
            .await;
        prepared.push(Prepared {
            media_type: "movie",
            media_id: *id,
            episode_ids: Vec::new(),
            path: format!("{PREFIX}{}", movie.file.path),
            size: movie.file.size,
            parsed,
            score,
        });
    }
    // A multi-episode file may arrive once with a range or once per episode:
    // either way it becomes one file linked to all of its episodes.
    let mut episode_file_index: HashMap<String, usize> = HashMap::new();
    for item in &report.episodes {
        let Some((series_id, name, original, year)) = series_rows.get(&item.series_tmdb_id) else {
            ignored += 1;
            continue;
        };
        let covered: Vec<i64> = (item.episode..=item.episode_end.unwrap_or(item.episode))
            .filter_map(|episode| {
                episode_ids
                    .get(&(*series_id, item.season, episode))
                    .copied()
            })
            .collect();
        if covered.is_empty() {
            ignored += 1;
            continue;
        }
        let path = format!("{PREFIX}{}", item.file.path);
        if let Some(&index) = episode_file_index.get(&path) {
            let existing = &mut prepared[index];
            if existing.media_id == *series_id {
                for id in covered {
                    if !existing.episode_ids.contains(&id) {
                        existing.episode_ids.push(id);
                    }
                }
            } else {
                ignored += 1;
            }
            continue;
        }
        let parsed = releases::parse(&descriptor(&item.file));
        let profile_id = series::effective_profile_for_episode(
            state,
            *series_id,
            item.season,
            Some(item.episode),
        )
        .await
        .unwrap_or(None);
        let score = profile_cache
            .score(
                state,
                profile_id,
                &parsed,
                name,
                original.as_deref(),
                *year,
                "series",
                Some(item.season),
            )
            .await;
        episode_file_index.insert(path.clone(), prepared.len());
        prepared.push(Prepared {
            media_type: "series",
            media_id: *series_id,
            episode_ids: covered,
            path,
            size: item.file.size,
            parsed,
            score,
        });
    }

    let previous: HashMap<String, i64> = sqlx::query_as::<_, (String, i64)>(
        "SELECT path,id FROM media_files WHERE discovered_by='cinetta' AND file_exists=1",
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?
    .into_iter()
    .collect();
    let reported: HashSet<&str> = prepared.iter().map(|p| p.path.as_str()).collect();
    let offline = offline_roots(&report.roots, previous.keys(), &reported);
    let mut kept_offline = 0usize;
    let mut removed: Vec<i64> = Vec::new();
    for (path, id) in &previous {
        if reported.contains(path.as_str()) {
            continue;
        }
        let real = path.trim_start_matches(PREFIX);
        if offline
            .iter()
            .any(|root| library_roots::contains_path(root, real))
        {
            kept_offline += 1;
        } else if report.allow_large_removal
            && report
                .roots
                .iter()
                .any(|root| root.available && library_roots::contains_path(&root.path, real))
        {
            removed.push(*id);
        }
    }

    let mut tx = state.db.begin().await.map_err(internal)?;
    let mut affected_episodes: HashSet<i64> = HashSet::new();
    for id in &removed {
        sqlx::query("UPDATE media_files SET file_exists=0 WHERE id=?")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        for episode in sqlx::query_scalar::<_, i64>(
            "SELECT episode_id FROM episode_files WHERE media_file_id=?",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await
        .map_err(internal)?
        {
            affected_episodes.insert(episode);
        }
    }
    let (mut movie_files, mut episode_files) = (0usize, 0usize);
    for item in &prepared {
        let quality_json = serde_json::to_string(&item.parsed).unwrap_or_else(|_| "{}".into());
        let file_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO media_files(
              media_type,media_id,download_job_id,path,size_bytes,quality_json,source_release,
              resolution,source,codec,hdr,audio,language,quality_score,file_exists,discovered_by,verified_at
            ) VALUES(?,?,NULL,?,?,?,?,?,?,?,?,?,?,?,1,'cinetta',CURRENT_TIMESTAMP)
            ON CONFLICT(path) DO UPDATE SET
              media_type=excluded.media_type,media_id=excluded.media_id,size_bytes=excluded.size_bytes,
              quality_json=excluded.quality_json,source_release=excluded.source_release,
              resolution=excluded.resolution,source=excluded.source,codec=excluded.codec,hdr=excluded.hdr,
              audio=excluded.audio,language=excluded.language,quality_score=excluded.quality_score,
              download_job_id=CASE WHEN media_files.media_type=excluded.media_type AND media_files.media_id=excluded.media_id
                                   THEN media_files.download_job_id END,
              file_exists=1,verified_at=CURRENT_TIMESTAMP
            RETURNING id
        "#,
        )
        .bind(item.media_type)
        .bind(item.media_id)
        .bind(&item.path)
        .bind(item.size)
        .bind(&quality_json)
        .bind(&item.parsed.name)
        .bind(&item.parsed.resolution)
        .bind(&item.parsed.source)
        .bind(&item.parsed.codec)
        .bind(&item.parsed.hdr)
        .bind(&item.parsed.audio)
        .bind(&item.parsed.language)
        .bind(item.score)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal)?;
        // A file Cinetta re-identified as other episodes must not keep
        // marking the old ones as present; every episode it still contains
        // stays linked.
        for old in sqlx::query_scalar::<_, i64>(
            "SELECT episode_id FROM episode_files WHERE media_file_id=?",
        )
        .bind(file_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(internal)?
        {
            if !item.episode_ids.contains(&old) {
                sqlx::query("DELETE FROM episode_files WHERE media_file_id=? AND episode_id=?")
                    .bind(file_id)
                    .bind(old)
                    .execute(&mut *tx)
                    .await
                    .map_err(internal)?;
                affected_episodes.insert(old);
            }
        }
        for episode_id in &item.episode_ids {
            sqlx::query(
                "INSERT INTO episode_files(episode_id,media_file_id) VALUES(?,?) ON CONFLICT(episode_id,media_file_id) DO NOTHING",
            )
            .bind(episode_id)
            .bind(file_id)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
            affected_episodes.insert(*episode_id);
        }
        if item.episode_ids.is_empty() {
            movie_files += 1;
        } else {
            episode_files += 1;
        }
    }
    // Only episodes whose Cinetta links changed are recomputed: a season pack
    // import can mark an episode present without a per-file link, and a full
    // recompute on every report would keep erasing that.
    for episode in &affected_episodes {
        sqlx::query("UPDATE series_episodes SET has_file=EXISTS(SELECT 1 FROM episode_files ef JOIN media_files mf ON mf.id=ef.media_file_id WHERE ef.episode_id=series_episodes.id AND mf.file_exists=1),updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(episode)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
    }
    if !external {
        link_external_jobs(&mut tx, &mappings).await?;
    }
    sqlx::query("INSERT INTO settings(key,value) VALUES('library.last_external_report',strftime('%Y-%m-%dT%H:%M:%SZ','now')) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
        .execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    // Inventory is informational only: no deletion proposals or filename-based
    // ownership inference are sent back to Cinetta in external mode.
    let superseded = if external {
        Vec::new()
    } else {
        superseded(state).await?
    };
    history::record(
        &state.db,
        "library.cinetta",
        "Cinetta",
        Some(&format!(
            "{movie_files} movie files · {episode_files} episode files · {ignored} not managed by Oberiz · {} missing · {kept_offline} kept (offline folder)",
            removed.len()
        )),
        "info",
    )
    .await;
    Ok(LibraryReportResult {
        movie_files,
        episode_files,
        ignored_files: ignored,
        missing_files: removed.len(),
        kept_offline_files: kept_offline,
        superseded,
    })
}

/// Roots whose previously reported files must be kept as they are: those the
/// client says it cannot read, and those it says it can read but that came
/// back with no files at all although they had some before — an empty mount
/// point looks exactly like that (library scans refuse it the same way).
fn offline_roots<'a>(
    roots: &'a [ReportedRoot],
    previous: impl Iterator<Item = &'a String> + Clone,
    reported: &HashSet<&str>,
) -> Vec<&'a str> {
    roots
        .iter()
        .filter(|root| {
            let had_files = previous.clone().any(|path| {
                library_roots::contains_path(&root.path, path.trim_start_matches(PREFIX))
            });
            let has_files = reported.iter().any(|path| {
                library_roots::contains_path(&root.path, path.trim_start_matches(PREFIX))
            });
            !root.available || (had_files && !has_files)
        })
        .map(|root| root.path.as_str())
        .collect()
}

/// Links files only beneath the download's full, explicitly mapped source
/// path. A matching basename on a different disk is not ownership evidence.
/// Only downloads completed while Oberiz was not managing files qualify: in
/// any other mode the import already recorded the file itself.
async fn link_external_jobs(
    tx: &mut sqlx::SqliteConnection,
    mappings: &[library_roots::PathMapping],
) -> Result<(), Error> {
    let jobs = sqlx::query_as::<_, (i64, String, i64, Option<i32>, Option<i32>, String)>(
        "SELECT id,media_type,media_id,season_number,episode_number,COALESCE(source_path,'') FROM download_jobs WHERE import_method='external' AND imported_at IS NOT NULL ORDER BY id DESC",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(internal)?;
    if jobs.is_empty() {
        return Ok(());
    }
    let files = sqlx::query_as::<_, (i64, String, i64, String)>(
        "SELECT id,media_type,media_id,path FROM media_files WHERE discovered_by='cinetta' AND file_exists=1 AND download_job_id IS NULL",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(internal)?;
    for (file_id, media_type, media_id, path) in files {
        let real = path.trim_start_matches(PREFIX);
        let job = jobs.iter().find(|(_, job_type, job_media, _, _, source)| {
            *job_type == media_type
                && *job_media == media_id
                && !source.is_empty()
                && library_roots::contains_path(&library_roots::map_path(source, mappings), real)
        });
        if let Some((job_id, ..)) = job {
            sqlx::query("UPDATE media_files SET download_job_id=? WHERE id=?")
                .bind(job_id)
                .bind(file_id)
                .execute(&mut *tx)
                .await
                .map_err(internal)?;
        }
    }
    Ok(())
}

/// Older versions made redundant by an Oberiz upgrade. For each movie or
/// episode, the newest file produced by an Oberiz download replaces any other
/// reported file that scores lower and is not itself a newer download.
/// Oberiz only lists them; deleting is Cinetta's opt-in decision.
async fn superseded(state: &AppState) -> Result<Vec<Superseded>, Error> {
    type FileRow = (
        i64,
        String,
        i64,
        Option<i64>,
        String,
        i32,
        Option<i64>,
        Option<i32>,
        Option<i32>,
        i64,
    );
    let rows = sqlx::query_as::<_, FileRow>(
        r#"
        SELECT mf.id, mf.media_type, mf.media_id, ef.episode_id, mf.path, mf.quality_score, mf.download_job_id,
               se.season_number, se.episode_number,
               CASE WHEN mf.media_type='movie' THEN (SELECT tmdb_id FROM movies WHERE id=mf.media_id)
                    ELSE (SELECT tmdb_id FROM series WHERE id=mf.media_id) END AS tmdb_id
        FROM media_files mf
        LEFT JOIN episode_files ef ON ef.media_file_id=mf.id
        LEFT JOIN series_episodes se ON se.id=ef.episode_id
        WHERE mf.discovered_by='cinetta' AND mf.file_exists=1
          AND (mf.media_type='movie' OR ef.episode_id IS NOT NULL)
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal)?;
    let mut groups: HashMap<(String, i64, Option<i64>), Vec<&FileRow>> = HashMap::new();
    for row in &rows {
        groups
            .entry((row.1.clone(), row.2, row.3))
            .or_default()
            .push(row);
    }
    let mut out = Vec::new();
    for files in groups.values() {
        let Some(newest) = files.iter().filter(|f| f.6.is_some()).max_by_key(|f| f.6) else {
            continue;
        };
        for old in files.iter().filter(|f| f.0 != newest.0) {
            if old.5 < newest.5 && old.6.is_none_or(|job| Some(job) < newest.6) {
                out.push(Superseded {
                    media_type: old.1.clone(),
                    tmdb_id: old.9,
                    season: old.7,
                    episode: old.8,
                    path: old.4.trim_start_matches(PREFIX).to_string(),
                    replacement_path: newest.4.trim_start_matches(PREFIX).to_string(),
                });
            }
        }
    }
    // A multi-episode file is redundant only when every linked episode has
    // a replacement; replacing E01 must not nominate an E01-E02 pack.
    let mut coverage = HashMap::<String, usize>::new();
    let mut replacements = HashMap::<String, usize>::new();
    for row in &rows {
        *coverage
            .entry(row.4.trim_start_matches(PREFIX).to_owned())
            .or_default() += 1;
    }
    for candidate in &out {
        *replacements.entry(candidate.path.clone()).or_default() += 1;
    }
    out.retain(|candidate| replacements.get(&candidate.path) == coverage.get(&candidate.path));
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

#[derive(Default)]
struct ProfileCache {
    profiles: HashMap<i64, Option<(profiles::QualityProfile, Option<profiles::LanguageProfile>)>>,
}

impl ProfileCache {
    /// Same stable scale as imports and scans (`importer::stable_quality_score`):
    /// title match plus profile rules, without seeders, so a reported file and
    /// a future search candidate are compared on equal terms.
    #[allow(clippy::too_many_arguments)]
    async fn score(
        &mut self,
        state: &AppState,
        profile_id: Option<i64>,
        parsed: &releases::ParsedRelease,
        title: &str,
        original: Option<&str>,
        year: Option<i32>,
        media_type: &str,
        season: Option<i32>,
    ) -> i32 {
        let Some(profile_id) = profile_id else {
            return parsed.score;
        };
        if let std::collections::hash_map::Entry::Vacant(slot) = self.profiles.entry(profile_id) {
            let loaded = match profiles::get_quality_profile_by_id(&state.db, profile_id).await {
                Ok(profile) => {
                    let language = profiles::language_for_profile(&state.db, &profile)
                        .await
                        .ok()
                        .flatten();
                    Some((profile, language))
                }
                Err(_) => None,
            };
            slot.insert(loaded);
        }
        let Some(Some((profile, language))) = self.profiles.get(&profile_id) else {
            return parsed.score;
        };
        let candidate = crate::cardigann::ReleaseResult {
            indexer_id: String::new(),
            indexer_name: String::new(),
            title: parsed.name.clone(),
            details_url: None,
            download_url: None,
            size_bytes: None,
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
        let evaluation = profiles::evaluate_release(
            &candidate,
            profile,
            language.as_ref(),
            title,
            original,
            year,
            media_type,
            season,
        );
        evaluation.match_score + evaluation.profile_score
    }
}

static RESOLUTION_TOKENS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)\b(2160p|1080p|720p|480p)\b").unwrap());
static CODEC_TOKENS: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"(?i)\b(x\.?26[45]|h\.?26[45]|hevc|avc|av1)\b").unwrap());

/// The text a reported file is scored from: its file name, with resolution
/// and video codec replaced by what Cinetta measured (ffprobe is more reliable
/// than a name, which can be wrong or missing them), and HDR or audio added
/// when the name says nothing about them. Source (REMUX, BluRay, WEB-DL…) can
/// only come from the name.
fn descriptor(file: &ReportedFile) -> String {
    let mut text = last_component(&file.path).unwrap_or_default();
    if let Some(resolution) = resolution_token(file.width, file.height) {
        text = format!("{} {resolution}", RESOLUTION_TOKENS.replace_all(&text, " "));
    }
    if let Some(codec) = codec_token(file.video_codec.as_deref()) {
        text = format!("{} {codec}", CODEC_TOKENS.replace_all(&text, " "));
    }
    let parsed = releases::parse(&text);
    if parsed.hdr.is_none()
        && file
            .hdr
            .as_deref()
            .is_some_and(|hdr| !hdr.trim().is_empty() && hdr != "0")
    {
        text.push_str(" HDR");
    }
    if parsed.audio.is_none()
        && let Some(audio) = audio_token(file.audio_codec.as_deref())
    {
        text.push(' ');
        text.push_str(audio);
    }
    text
}

fn resolution_token(width: Option<i64>, height: Option<i64>) -> Option<&'static str> {
    let (width, height) = (width.unwrap_or(0), height.unwrap_or(0));
    // Width matters: a letterboxed 4K film is 3840×1600, not 2160 tall.
    match (width, height) {
        (0, 0) => None,
        (w, h) if w >= 3200 || h >= 1800 => Some("2160p"),
        (w, h) if w >= 1700 || h >= 900 => Some("1080p"),
        (w, h) if w >= 1100 || h >= 650 => Some("720p"),
        _ => Some("480p"),
    }
}

fn codec_token(codec: Option<&str>) -> Option<&'static str> {
    match codec?.trim().to_ascii_lowercase().as_str() {
        "hevc" | "h265" => Some("HEVC"),
        "h264" | "avc" => Some("H264"),
        "av1" => Some("AV1"),
        _ => None,
    }
}

fn audio_token(codec: Option<&str>) -> Option<&'static str> {
    match codec?.trim().to_ascii_lowercase().as_str() {
        "truehd" => Some("TRUEHD"),
        "eac3" => Some("EAC3"),
        "dts" => Some("DTS"),
        "flac" => Some("FLAC"),
        "aac" => Some("AAC"),
        _ => None,
    }
}

#[cfg(test)]
fn components(path: &str) -> Vec<String> {
    path.trim_start_matches(PREFIX)
        .split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn last_component(path: &str) -> Option<String> {
    path.trim_start_matches(PREFIX)
        .split(['/', '\\'])
        .rfind(|part| !part.is_empty())
        .map(str::to_string)
}

fn internal(error: impl std::fmt::Display) -> Error {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str) -> ReportedFile {
        ReportedFile {
            path: path.into(),
            size: None,
            width: None,
            height: None,
            video_codec: None,
            hdr: None,
            audio_codec: None,
        }
    }

    #[test]
    fn measured_resolution_and_codec_replace_what_the_name_claims() {
        let mut reported = file("/movies/Film (2020)/Film.2020.1080p.BluRay.x264.mkv");
        reported.width = Some(3840);
        reported.height = Some(1600);
        reported.video_codec = Some("hevc".into());
        let parsed = releases::parse(&descriptor(&reported));
        assert_eq!(parsed.resolution.as_deref(), Some("2160P"));
        assert_eq!(parsed.codec.as_deref(), Some("HEVC"));
        assert_eq!(parsed.source.as_deref(), Some("BluRay"));
    }

    #[test]
    fn hdr_and_audio_are_added_only_when_the_name_is_silent() {
        let mut reported = file(r"\\nas\movies\Film (2020).mkv");
        reported.hdr = Some("HDR".into());
        reported.audio_codec = Some("truehd".into());
        let parsed = releases::parse(&descriptor(&reported));
        assert_eq!(parsed.hdr.as_deref(), Some("HDR"));
        assert_eq!(parsed.audio.as_deref(), Some("TrueHD"));

        let mut named = file("/m/Film.2020.2160p.DV.DTS-HD.mkv");
        named.hdr = Some("HDR".into());
        named.audio_codec = Some("aac".into());
        let parsed = releases::parse(&descriptor(&named));
        assert_eq!(parsed.hdr.as_deref(), Some("Dolby Vision"));
        assert_eq!(parsed.audio.as_deref(), Some("DTS-HD"));
    }

    async fn state() -> AppState {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        let db = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        sqlx::query("INSERT INTO movies(id,tmdb_id,title,year) VALUES(1,603,'Matrix',1999)")
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("INSERT INTO series(id,tmdb_id,name,year) VALUES(1,1399,'Silo',2023)")
            .execute(&db)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO series_seasons(id,series_id,season_number,name) VALUES(1,1,1,'S1')",
        )
        .execute(&db)
        .await
        .unwrap();
        sqlx::query("INSERT INTO series_episodes(id,series_id,season_id,season_number,episode_number,name) VALUES(1,1,1,1,1,'E1'),(2,1,1,1,2,'E2')")
            .execute(&db)
            .await
            .unwrap();
        AppState {
            db,
            http: reqwest::Client::new(),
            automation_runtime: Default::default(),
            download_client: Arc::new(crate::download_client::QBittorrentClient),
            torrent_list_cache: Arc::new(Mutex::new(None)),
            grab_lock: Arc::new(Mutex::new(())),
        }
    }

    fn movie(tmdb_id: i64, path: &str, height: i64) -> ReportedMovie {
        let mut file = file(path);
        file.width = Some(height * 16 / 9);
        file.height = Some(height);
        ReportedMovie { tmdb_id, file }
    }

    async fn available(state: &AppState) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM media_files WHERE media_type='movie' AND media_id=1 AND file_exists=1")
            .fetch_one(&state.db)
            .await
            .unwrap()
    }

    fn empty_report() -> LibraryReport {
        LibraryReport {
            movies: vec![],
            episodes: vec![],
            roots: vec![],
            allow_large_removal: false,
        }
    }

    #[tokio::test]
    async fn switching_to_external_waits_for_active_media_operations() {
        let state = state().await;
        let guard = crate::importer::MEDIA_OPERATION_GATE.lock().await;
        let payload =
            serde_json::from_value(serde_json::json!({"library_source":"cinetta"})).unwrap();
        let mut update = tokio::spawn({
            let state = state.clone();
            async move { crate::settings::update_settings(State(state), Json(payload)).await }
        });
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), &mut update)
                .await
                .is_err()
        );
        assert!(!crate::importer::external_mode(&state).await.unwrap());
        drop(guard);
        assert!(update.await.unwrap().is_ok());
        assert!(crate::importer::external_mode(&state).await.unwrap());
    }

    #[tokio::test]
    async fn authorization_precedes_json_and_busy_reports_are_not_queued() {
        let state = state().await;
        crate::settings::set_value(&state.db, "api.enabled", "true")
            .await
            .unwrap();
        crate::settings::set_value(
            &state.db,
            "api.key_hash",
            &crate::settings::hash_api_key("synthetic-test-key"),
        )
        .await
        .unwrap();
        let app = axum::Router::new()
            .route("/inventory", axum::routing::put(report))
            .layer(axum::extract::DefaultBodyLimit::max(1024))
            .with_state(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/inventory", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let response = client
            .put(&url)
            .header("content-type", "application/json")
            .body("not-json".repeat(1024))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let response = client
            .put(&url)
            .header("x-api-key", "synthetic-test-key")
            .header("content-type", "application/json")
            .body("not-json")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let response = client
            .put(&url)
            .header("x-api-key", "synthetic-test-key")
            .header("content-type", "application/json")
            .body(" ".repeat(2048))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        let held = REPORT_SLOTS.clone().acquire_many_owned(2).await.unwrap();
        let response = client
            .put(&url)
            .header("x-api-key", "synthetic-test-key")
            .header("content-type", "application/json")
            .body("not-json")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        drop(held);
        let response = client
            .put(&url)
            .header("x-api-key", "synthetic-test-key")
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CONFLICT);
        crate::settings::set_value(&state.db, "library.source", "cinetta")
            .await
            .unwrap();
        let response = client
            .put(&url)
            .header("x-api-key", "synthetic-test-key")
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            crate::settings::get_value(&state.db, "library.last_external_report")
                .await
                .unwrap()
                .is_some()
        );
        server.abort();
    }

    #[test]
    fn invalid_ranges_metadata_roots_and_conflicting_paths_are_rejected() {
        let mut report = empty_report();
        report
            .episodes
            .push(episode(i32::MIN, Some(i32::MAX), "/file.mkv"));
        assert_eq!(validate(&report).unwrap_err().0, StatusCode::BAD_REQUEST);
        report.episodes.clear();
        report.movies.push(movie(603, "/same.mkv", 1080));
        report.episodes.push(episode(1, None, "/same.mkv"));
        assert!(validate(&report).is_err());
        report.episodes.clear();
        report.movies[0].file.size = Some(-1);
        assert!(validate(&report).is_err());
        report.movies.clear();
        report.roots = (0..=MAX_ROOTS)
            .map(|i| ReportedRoot {
                path: format!("/root/{i}"),
                available: true,
            })
            .collect();
        assert_eq!(
            validate(&report).unwrap_err().0,
            StatusCode::PAYLOAD_TOO_LARGE
        );
    }

    #[tokio::test]
    async fn reidentifying_a_series_file_as_a_movie_removes_old_episode_links() {
        let state = state().await;
        let mut report = empty_report();
        report
            .episodes
            .push(episode(1, Some(2), "/media/reidentified.mkv"));
        apply(&state, report).await.unwrap();
        assert_eq!(episodes_with_file(&state).await, vec![1, 2]);
        let mut report = empty_report();
        report
            .movies
            .push(movie(603, "/media/reidentified.mkv", 1080));
        apply(&state, report).await.unwrap();
        assert!(episodes_with_file(&state).await.is_empty());
        let links: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM episode_files")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(links, 0);
    }

    #[tokio::test]
    async fn external_mode_blocks_old_jobs_and_local_discovery_and_returns_no_deletion_advice() {
        let state = state().await;
        crate::settings::set_value(&state.db, "library.source", "cinetta")
            .await
            .unwrap();
        crate::settings::set_value(&state.db, "movies_path", "/unreachable/synthetic-share")
            .await
            .unwrap();
        assert_eq!(
            crate::importer::reseed(State(state.clone()), axum::extract::Path(123))
                .await
                .unwrap_err()
                .0,
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            crate::library::scan_media(&state, "movie")
                .await
                .unwrap_err()
                .0,
            StatusCode::FORBIDDEN
        );
        crate::library::ensure_discovered(&state, "movie", 1)
            .await
            .unwrap();
        assert!(
            crate::library::root_status(State(state.clone()))
                .await
                .unwrap()
                .0
                .is_empty()
        );
        assert!(
            crate::library_roots::mapping_suggestions(State(state.clone()))
                .await
                .unwrap()
                .0
                .is_empty()
        );
        let mut report = empty_report();
        report.movies.push(movie(603, "/media/Matrix.mkv", 1080));
        assert!(apply(&state, report).await.unwrap().superseded.is_empty());
    }

    #[tokio::test]
    async fn replacing_one_episode_does_not_supersede_a_multi_episode_file() {
        let state = state().await;
        let mut report = empty_report();
        let mut old = episode(1, Some(2), "/tv/Silo/old.720p.mkv");
        old.file.height = Some(720);
        let mut new = episode(1, None, "/tv/Silo/new.1080p.mkv");
        new.file.height = Some(1080);
        report.episodes = vec![old, new];
        apply(&state, report).await.unwrap();
        sqlx::query("INSERT INTO download_jobs(id,media_type,media_id,release_title,indexer_id,status) VALUES(7,'series',1,'Synthetic','idx','seeding')").execute(&state.db).await.unwrap();
        sqlx::query("UPDATE media_files SET download_job_id=7 WHERE path LIKE '%new.1080p.mkv'")
            .execute(&state.db)
            .await
            .unwrap();
        assert!(superseded(&state).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_matching_basename_on_another_disk_does_not_claim_download_ownership() {
        let state = state().await;
        sqlx::query("INSERT INTO download_jobs(id,media_type,media_id,release_title,indexer_id,status,source_path,import_method,imported_at) VALUES(7,'movie',1,'Matrix','idx','seeding','/another-disk/Matrix','external',CURRENT_TIMESTAMP)").execute(&state.db).await.unwrap();
        let mut report = empty_report();
        report
            .movies
            .push(movie(603, "/media/Matrix/Matrix.mkv", 1080));
        apply(&state, report).await.unwrap();
        let owner: Option<i64> = sqlx::query_scalar("SELECT download_job_id FROM media_files")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(owner, None);
    }

    #[tokio::test]
    async fn reported_files_make_managed_titles_available_and_ignore_the_rest() {
        let state = state().await;
        let result = apply(
            &state,
            LibraryReport {
                movies: vec![
                    movie(603, "/media/Matrix (1999)/Matrix.mkv", 1080),
                    movie(999, "/media/Unmanaged (2000)/u.mkv", 1080),
                ],
                episodes: vec![ReportedEpisode {
                    series_tmdb_id: 1399,
                    season: 1,
                    episode: 2,
                    episode_end: None,
                    file: file("/tv/Silo/S01E02.mkv"),
                }],
                roots: vec![],
                allow_large_removal: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            (
                result.movie_files,
                result.episode_files,
                result.ignored_files
            ),
            (1, 1, 1)
        );
        assert_eq!(available(&state).await, 1);
        let episodes: Vec<(i64, bool)> =
            sqlx::query_as("SELECT id,has_file FROM series_episodes ORDER BY id")
                .fetch_all(&state.db)
                .await
                .unwrap();
        assert_eq!(episodes, vec![(1, false), (2, true)]);

        // An incomplete or empty report is not evidence of deletion.
        let result = apply(
            &state,
            LibraryReport {
                movies: vec![],
                episodes: vec![],
                roots: vec![],
                allow_large_removal: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(result.missing_files, 0);
        assert_eq!(available(&state).await, 1);
        let has_file: bool = sqlx::query_scalar("SELECT has_file FROM series_episodes WHERE id=2")
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert!(has_file);
    }

    #[tokio::test]
    async fn an_oberiz_upgrade_marks_the_older_lower_quality_version_as_superseded() {
        let state = state().await;
        apply(
            &state,
            LibraryReport {
                movies: vec![movie(603, "/media/Matrix (1999)/Matrix.720p.mkv", 720)],
                episodes: vec![],
                roots: vec![],
                allow_large_removal: false,
            },
        )
        .await
        .unwrap();
        sqlx::query("INSERT INTO download_jobs(id,media_type,media_id,release_title,indexer_id,status,source_path,import_method,imported_at) VALUES(7,'movie',1,'Matrix.1999.2160p.BluRay.x265','idx','seeding','/media/Matrix.1999.2160p.BluRay.x265','external',CURRENT_TIMESTAMP)")
            .execute(&state.db)
            .await
            .unwrap();
        let result = apply(
            &state,
            LibraryReport {
                movies: vec![
                    movie(603, "/media/Matrix (1999)/Matrix.720p.mkv", 720),
                    movie(
                        603,
                        "/media/Matrix.1999.2160p.BluRay.x265/Matrix.1999.2160p.BluRay.x265.mkv",
                        2160,
                    ),
                ],
                episodes: vec![],
                roots: vec![],
                allow_large_removal: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(result.superseded.len(), 1);
        assert_eq!(
            result.superseded[0].path,
            "/media/Matrix (1999)/Matrix.720p.mkv"
        );
        assert_eq!(result.superseded[0].tmdb_id, 603);
        assert!(result.superseded[0].replacement_path.contains("2160p"));
    }

    #[tokio::test]
    async fn removals_require_confirmation_and_a_readable_nonempty_root() {
        let state = state().await;
        let many = (0..30)
            .map(|i| movie(603, &format!("/media/Matrix/{i}.mkv"), 1080))
            .collect();
        apply(
            &state,
            LibraryReport {
                movies: many,
                episodes: vec![],
                roots: vec![],
                allow_large_removal: false,
            },
        )
        .await
        .unwrap();
        let preserved = apply(
            &state,
            LibraryReport {
                movies: vec![],
                episodes: vec![],
                roots: vec![],
                allow_large_removal: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(preserved.missing_files, 0);
        assert_eq!(available(&state).await, 30);
        apply(
            &state,
            LibraryReport {
                movies: vec![movie(603, "/media/Matrix/new.mkv", 1080)],
                episodes: vec![],
                roots: vec![ReportedRoot {
                    path: "/media".into(),
                    available: true,
                }],
                allow_large_removal: true,
            },
        )
        .await
        .unwrap();
        assert_eq!(available(&state).await, 1);
    }

    fn episode(number: i32, end: Option<i32>, path: &str) -> ReportedEpisode {
        ReportedEpisode {
            series_tmdb_id: 1399,
            season: 1,
            episode: number,
            episode_end: end,
            file: file(path),
        }
    }

    async fn episodes_with_file(state: &AppState) -> Vec<i64> {
        sqlx::query_scalar("SELECT id FROM series_episodes WHERE has_file=1 ORDER BY id")
            .fetch_all(&state.db)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn a_double_episode_file_keeps_both_episodes_linked() {
        for episodes in [
            // Reported once per episode…
            vec![
                episode(1, None, "/tv/Silo/S01E01E02.mkv"),
                episode(2, None, "/tv/Silo/S01E01E02.mkv"),
            ],
            // …or once with a range.
            vec![episode(1, Some(2), "/tv/Silo/S01E01E02.mkv")],
        ] {
            let state = state().await;
            let result = apply(
                &state,
                LibraryReport {
                    movies: vec![],
                    episodes,
                    roots: vec![],
                    allow_large_removal: false,
                },
            )
            .await
            .unwrap();
            assert_eq!(result.episode_files, 1);
            assert_eq!(episodes_with_file(&state).await, vec![1, 2]);
        }
    }

    #[tokio::test]
    async fn files_on_an_offline_or_suddenly_empty_root_are_kept() {
        let state = state().await;
        let library = || LibraryReport {
            movies: vec![
                movie(603, "/disk-a/Matrix (1999)/Matrix.mkv", 1080),
                movie(603, "/disk-b/Matrix (1999)/Matrix.mkv", 1080),
            ],
            episodes: vec![],
            roots: vec![
                ReportedRoot {
                    path: "/disk-a".into(),
                    available: true,
                },
                ReportedRoot {
                    path: "/disk-b".into(),
                    available: true,
                },
            ],
            allow_large_removal: false,
        };
        apply(&state, library()).await.unwrap();
        assert_eq!(available(&state).await, 2);

        // Disk A reported offline, disk B mounted but empty: nothing is lost.
        let mut report = library();
        report.movies.clear();
        report.roots[0].available = false;
        let result = apply(&state, report).await.unwrap();
        assert_eq!((result.missing_files, result.kept_offline_files), (0, 2));
        assert_eq!(available(&state).await, 2);

        // Disk B back with its file, disk A still offline: still nothing lost.
        let mut report = library();
        report.movies.remove(0);
        report.roots[0].available = false;
        let result = apply(&state, report).await.unwrap();
        assert_eq!((result.missing_files, result.kept_offline_files), (0, 1));
        assert_eq!(available(&state).await, 2);

        // Both disks readable and the file really gone from disk A.
        let mut report = library();
        report.movies.remove(0);
        report
            .movies
            .push(movie(603, "/disk-a/Other/Other.mkv", 1080));
        report.allow_large_removal = true;
        let result = apply(&state, report).await.unwrap();
        assert_eq!(result.missing_files, 1);
    }

    #[test]
    fn path_components_ignore_the_prefix_and_separators() {
        assert_eq!(
            components(r"cinetta:\\nas\Movies\Film (2020)\film.mkv"),
            vec!["nas", "movies", "film (2020)", "film.mkv"]
        );
        assert_eq!(
            last_component("/downloads/Film.2020.2160p/").as_deref(),
            Some("Film.2020.2160p")
        );
    }
}
