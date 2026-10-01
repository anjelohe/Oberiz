mod state;

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use state::{record_error, record_series_target_error, touch_search};
use std::{
    collections::BTreeMap,
    sync::{LazyLock, atomic::Ordering},
};
use tokio::time::{Duration, sleep};

use crate::{
    AppState,
    cardigann::ReleaseResult,
    history, importer, profiles,
    search_api::{self, GrabRequest},
    series, settings,
};

pub use state::AutomationRuntime;

/// A single monitored item can search once (a movie) or, for a series with no
/// pack match, once per missing episode — so a handful of series with long
/// backlogs can otherwise queue up hundreds of indexer sweeps in one cycle.
/// Capping searches per cycle keeps that bounded; anything past the cap is
/// left for the next cycle instead of being lost (no automation_state update
/// happens for a skipped item, so it stays eligible immediately).
const MAX_SEARCHES_PER_CYCLE: usize = 40;

#[derive(Debug, Clone)]
pub(crate) struct RssRelease {
    pub indexer_id: String,
    pub indexer_name: String,
    pub guid: String,
    pub title: String,
    pub download_url: Option<String>,
    pub details_url: Option<String>,
    pub published: Option<String>,
}

fn normalized_title(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

fn release_for_rss(item: &RssRelease) -> ReleaseResult {
    let parsed = crate::releases::parse(&item.title);
    ReleaseResult {
        indexer_id: item.indexer_id.clone(),
        indexer_name: item.indexer_name.clone(),
        title: item.title.clone(),
        details_url: item.details_url.clone(),
        download_url: item.download_url.clone(),
        size_bytes: None,
        seeders: None,
        leechers: None,
        category: None,
        published: item.published.clone(),
        score: parsed.score,
        resolution: parsed.resolution,
        source: parsed.source,
        codec: parsed.codec,
        hdr: parsed.hdr,
        audio: parsed.audio,
        language: parsed.language,
        base_score: parsed.score,
        profile_score: 0,
        match_score: 0,
        accepted: false,
        reasons: vec![],
        rejection_reasons: vec![],
    }
}

/// Evaluates one new RSS release against monitored media. It intentionally accepts only one
/// target: movie first, then Complete Series, season pack, then episode.
pub(crate) async fn process_rss_release(
    state: &AppState,
    item: &RssRelease,
) -> Result<String, String> {
    let release_title = normalized_title(&item.title);
    if release_title.len() < 4 {
        return Ok("ignored: title too short".into());
    }

    // Tracks the most relevant non-grab outcome across every monitored title this
    // release's own text happens to match, instead of returning on the very first
    // one. A short/common title ("It", "Dune") can substring-match the wrong entry
    // first; returning immediately there would silently stop the release from ever
    // being evaluated against the title it actually corresponds to.
    let mut last_outcome = "ignored: no monitored match".to_string();
    let movies=sqlx::query_as::<_,(i64,String,Option<String>,Option<i32>,Option<i64>)>(
        "SELECT id,title,original_title,year,quality_profile_id FROM movies WHERE monitored=1 ORDER BY id"
    ).fetch_all(&state.db).await.map_err(|e|e.to_string())?;
    for (id, title, original, year, profile_id) in movies {
        let main = normalized_title(&title);
        let alternate = original
            .as_deref()
            .map(normalized_title)
            .unwrap_or_default();
        if !release_title.contains(&main)
            && (alternate.is_empty() || !release_title.contains(&alternate))
        {
            continue;
        }
        let Some(profile_id) = profile_id else {
            last_outcome = format!("skipped: {title} has no quality profile");
            continue;
        };
        let profile = profiles::get_quality_profile_by_id(&state.db, profile_id)
            .await
            .map_err(|e| e.1)?;
        if !profile.enabled {
            last_outcome = format!("skipped: {title} profile disabled");
            continue;
        }
        let language = profiles::language_for_profile(&state.db, &profile)
            .await
            .map_err(|e| e.1)?;
        let mut candidate = release_for_rss(item);
        let evaluation = profiles::evaluate_release(
            &candidate,
            &profile,
            language.as_ref(),
            &title,
            original.as_deref(),
            year,
            "movie",
            None,
        );
        candidate.base_score = candidate.score;
        candidate.profile_score = evaluation.profile_score;
        candidate.match_score = evaluation.match_score;
        candidate.score = evaluation.total_score;
        candidate.accepted = evaluation.accepted;
        candidate.rejection_reasons = evaluation.rejection_reasons.clone();
        if !candidate.accepted {
            last_outcome = format!("rejected: {}", candidate.rejection_reasons.join(", "));
            continue;
        }
        let current=sqlx::query_scalar::<_,Option<i32>>("SELECT MAX(quality_score) FROM media_files WHERE media_type='movie' AND media_id=? AND file_exists=1")
            .bind(id).fetch_one(&state.db).await.unwrap_or(None);
        // Compared on the match+profile-rule scale, not candidate.score (which
        // also folds in seeders) — `current` is the file's quality_score,
        // persisted on that same stable scale (see importer::stable_quality_score).
        let candidate_quality = candidate.match_score + candidate.profile_score;
        if current.is_some_and(|score| !profile.upgrade_allowed || candidate_quality <= score + 25)
        {
            last_outcome = "skipped: no quality upgrade".into();
            continue;
        }
        // Held through the grab so a concurrent automation cycle re-checking
        // the same movie can't slip in between this check and the insert.
        let _lock = state.grab_lock.lock().await;
        if already_grabbed(state, "movie", id, None, None).await {
            last_outcome = "skipped: equivalent job already active".into();
            continue;
        }
        grab_rss(
            state,
            &candidate,
            "movie",
            id,
            Some(profile.id),
            None,
            None,
            false,
        )
        .await?;
        history::record(
            &state.db,
            "rss.grabbed",
            &title,
            Some(&format!(
                "{} · score {} · {}",
                item.title, candidate.score, item.indexer_name
            )),
            "info",
        )
        .await;
        return Ok(format!("grabbed: movie {title}"));
    }

    let shows = sqlx::query_as::<_, (i64, String, Option<String>, Option<i64>)>(
        "SELECT id,name,original_name,quality_profile_id FROM series WHERE monitored=1 ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| e.to_string())?;
    for (id, title, original, profile_id) in shows {
        let main = normalized_title(&title);
        let alternate = original
            .as_deref()
            .map(normalized_title)
            .unwrap_or_default();
        if !release_title.contains(&main)
            && (alternate.is_empty() || !release_title.contains(&alternate))
        {
            continue;
        }
        let Some(series_profile_id) = profile_id else {
            last_outcome = format!("skipped: {title} has no quality profile");
            continue;
        };
        let series_profile = profiles::get_quality_profile_by_id(&state.db, series_profile_id)
            .await
            .map_err(|e| e.1)?;
        if !series_profile.enabled {
            last_outcome = format!("skipped: {title} profile disabled");
            continue;
        }
        let missing=sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND monitored=1 AND has_file=0 AND (air_date IS NULL OR air_date<=date('now','localtime'))")
            .bind(id).fetch_one(&state.db).await.unwrap_or(0);
        if missing == 0 {
            last_outcome = "skipped: series is already complete".into();
            continue;
        }
        if is_complete_series_title(&item.title) && series_profile.rules.series_accept_complete {
            // Complete-series releases have no single season/episode to resolve
            // an override for, so the series-level profile is the right one.
            let language = profiles::language_for_profile(&state.db, &series_profile)
                .await
                .map_err(|e| e.1)?;
            let mut candidate = release_for_rss(item);
            let evaluation = profiles::evaluate_release(
                &candidate,
                &series_profile,
                language.as_ref(),
                &title,
                original.as_deref(),
                None,
                "series",
                None,
            );
            candidate.base_score = candidate.score;
            candidate.profile_score = evaluation.profile_score;
            candidate.match_score = evaluation.match_score;
            candidate.score = evaluation.total_score;
            candidate.accepted = evaluation.accepted;
            candidate.rejection_reasons = evaluation.rejection_reasons.clone();
            if !candidate.accepted {
                last_outcome = format!("rejected: {}", candidate.rejection_reasons.join(", "));
                continue;
            }
            let _lock = state.grab_lock.lock().await;
            if already_grabbed(state, "series", id, None, None).await {
                last_outcome = "skipped: equivalent job already active".into();
                continue;
            }
            grab_rss(
                state,
                &candidate,
                "series",
                id,
                Some(series_profile.id),
                None,
                None,
                true,
            )
            .await?;
            history::record(
                &state.db,
                "rss.complete_series_grabbed",
                &title,
                Some(&format!("{} · score {}", item.title, candidate.score)),
                "info",
            )
            .await;
            return Ok(format!("grabbed: complete series {title}"));
        }
        // Compiled once: this runs per RSS item, and re-compiling per call was
        // also silently broken — the doubled `\\d` matched a literal
        // backslash instead of a digit, so this never matched a real title.
        static SEASON_EPISODE: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"(?i)S(\d{1,2})(?:E(\d{1,3}))?").unwrap());
        let Some(caps) = SEASON_EPISODE.captures(&item.title) else {
            last_outcome = "skipped: series release has no season/episode target".into();
            continue;
        };
        let season = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0);
        let episode = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
        if season <= 0 {
            last_outcome = "skipped: invalid series target".into();
            continue;
        }
        let is_pack = episode.is_none();
        // Resolves the season/episode-specific profile override (if any)
        // instead of assuming the series-level default applies: RSS used to
        // evaluate and grab every release under the series' own profile even
        // when a season or episode had its own override configured, so a 4K
        // or strict-language override, say, could be silently bypassed by RSS
        // while the scheduled search correctly respected it.
        let effective_profile_id =
            series::effective_profile_for_episode(state, id, season, episode)
                .await
                .map_err(|e| e.1)?
                .unwrap_or(series_profile_id);
        let profile = if effective_profile_id == series_profile_id {
            series_profile
        } else {
            profiles::get_quality_profile_by_id(&state.db, effective_profile_id)
                .await
                .map_err(|e| e.1)?
        };
        if !profile.enabled {
            last_outcome = format!("skipped: {title} target profile disabled");
            continue;
        }
        if is_pack && !profile.rules.series_prefer_pack {
            last_outcome = "skipped: season packs disabled by profile".into();
            continue;
        }
        let language = profiles::language_for_profile(&state.db, &profile)
            .await
            .map_err(|e| e.1)?;
        let mut candidate = release_for_rss(item);
        let evaluation = profiles::evaluate_release(
            &candidate,
            &profile,
            language.as_ref(),
            &title,
            original.as_deref(),
            None,
            "series",
            Some(season),
        );
        candidate.base_score = candidate.score;
        candidate.profile_score = evaluation.profile_score;
        candidate.match_score = evaluation.match_score;
        candidate.score = evaluation.total_score;
        candidate.accepted = evaluation.accepted;
        candidate.rejection_reasons = evaluation.rejection_reasons.clone();
        if !candidate.accepted {
            last_outcome = format!("rejected: {}", candidate.rejection_reasons.join(", "));
            continue;
        }
        let exists = if let Some(ep) = episode {
            sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND season_number=? AND episode_number=? AND monitored=1 AND has_file=0")
                .bind(id).bind(season).bind(ep).fetch_one(&state.db).await.unwrap_or(0)>0
        } else {
            sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND season_number=? AND monitored=1 AND has_file=0")
                .bind(id).bind(season).fetch_one(&state.db).await.unwrap_or(0)>0
        };
        if !exists {
            last_outcome = "skipped: target is not wanted".into();
            continue;
        }
        let _lock = state.grab_lock.lock().await;
        if already_grabbed(state, "series", id, Some(season), episode).await {
            last_outcome = "skipped: equivalent job already active".into();
            continue;
        }
        grab_rss(
            state,
            &candidate,
            "series",
            id,
            Some(profile.id),
            Some(season),
            episode,
            is_pack,
        )
        .await?;
        let kind = if is_pack { "season pack" } else { "episode" };
        history::record(
            &state.db,
            "rss.series_grabbed",
            &title,
            Some(&format!(
                "{} · {} · score {}",
                item.title, kind, candidate.score
            )),
            "info",
        )
        .await;
        return Ok(format!("grabbed: {kind} {title}"));
    }
    Ok(last_outcome)
}

/// Whether this exact target has already been grabbed or is in flight.
/// 'cleaned' deliberately does NOT count as active here: this gate sits
/// behind a caller-side check ("do I already have this at a good enough
/// score/does it have a file") for every media type that reaches it, so once
/// a job is cleaned, whether to grab again is that upstream check's call —
/// treating 'cleaned' as eternally active here would instead block legitimate
/// quality upgrades forever, since the original (now finished) job never
/// stops "counting".
/// Whether an active job already covers this target. An exact (season,
/// episode) match isn't the only way that can be true for a series: a
/// season-pack job (episode_number NULL) already covers every episode in
/// that season, and a complete-series job (season_number NULL too) covers
/// everything. Checking only for an exact tuple match let RSS/scheduled/
/// manual grabs miss each other's scope entirely — an episode request next
/// to an in-flight pack for the same season, or either next to an in-flight
/// complete-series job, none of which share one (season, episode) tuple.
async fn already_grabbed(
    state: &AppState,
    media_type: &str,
    media_id: i64,
    season: Option<i32>,
    episode: Option<i32>,
) -> bool {
    sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM download_jobs
        WHERE media_type=? AND media_id=?
          AND status IN ('queued','downloading','completed','seeding')
          AND (
            (season_number IS ? AND episode_number IS ?)
            OR (season_number IS NULL AND episode_number IS NULL)
            OR (? IS NOT NULL AND season_number = ? AND episode_number IS NULL)
            OR (? IS NULL AND season_number = ? AND episode_number IS NOT NULL)
          )"#,
    )
    .bind(media_type)
    .bind(media_id)
    .bind(season)
    .bind(episode)
    .bind(episode)
    .bind(season)
    .bind(episode)
    .bind(season)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0)
        > 0
}

#[allow(clippy::too_many_arguments)]
async fn grab_rss(
    state: &AppState,
    candidate: &ReleaseResult,
    media_type: &str,
    media_id: i64,
    profile_id: Option<i64>,
    season: Option<i32>,
    episode: Option<i32>,
    is_pack: bool,
) -> Result<(), String> {
    let req = GrabRequest {
        indexer_id: candidate.indexer_id.clone(),
        indexer_name: Some(candidate.indexer_name.clone()),
        title: candidate.title.clone(),
        download_url: candidate.download_url.clone(),
        details_url: candidate.details_url.clone(),
        category: None,
        media_type: Some(media_type.into()),
        media_id: Some(media_id),
        profile_id,
        season_number: season,
        episode_number: episode,
        is_season_pack: Some(is_pack),
    };
    search_api::grab_internal(state, &req)
        .await
        .map_err(|(_, e)| e)
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SeriesStrategy {
    Complete,
    SeasonPack,
    Episode,
}

fn is_complete_series_title(title: &str) -> bool {
    regex::Regex::new(r"(?i)(?:\bCOMPLETE[\s._-]+(?:SERIES|COLLECTION|PACK)\b|\bTHE[\s._-]+COMPLETE[\s._-]+SERIES\b|\bFULL[\s._-]+SERIES\b)")
        .map(|re|re.is_match(title)).unwrap_or(false)
}

fn matches_series_target(title: &str, season_number: i32, episode_number: Option<i32>) -> bool {
    // "NxNN" (e.g. "2x05") is a release-naming convention some indexers still
    // use, and the local-file importer already recognizes it (see
    // importer/naming.rs) — but this function, which decides whether a search
    // result actually satisfies the target before grabbing it, only checked
    // for "sNNeNN". A `\b` before the season digit keeps this from matching
    // inside a resolution like "1920x1080".
    let season_x = regex::Regex::new(&format!(r"\b{season_number}x(\d{{1,3}})\b"))
        .ok()
        .and_then(|re| re.captures(title))
        .and_then(|caps| caps.get(1)?.as_str().parse::<i32>().ok());
    let normalized = title.to_lowercase();
    let season_token = format!("s{season_number:02}");
    let has_season = normalized.contains(&season_token) || season_x.is_some();
    if !has_season {
        return false;
    }
    match episode_number {
        Some(episode) => {
            normalized.contains(&format!("e{episode:02}")) || season_x == Some(episode)
        }
        None => {
            let looks_like_single_episode = season_x.is_some()
                || regex::Regex::new(r"(?i)S\d{1,2}E\d{1,3}")
                    .map(|re| re.is_match(&normalized))
                    .unwrap_or(false);
            !looks_like_single_episode
                || normalized.contains("season")
                || normalized.contains("complete")
                || normalized.contains("pack")
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AutomationItemState {
    pub media_type: String,
    pub media_id: i64,
    pub last_search_at: Option<String>,
    pub last_grab_at: Option<String>,
    pub last_grab_title: Option<String>,
    pub last_grab_score: Option<i32>,
    pub last_error: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct AutomationStatus {
    pub enabled: bool,
    pub interval_minutes: u64,
    pub monitored_movies: i64,
    pub monitored_series: i64,
    pub wanted_episodes: i64,
    pub states: Vec<AutomationItemState>,
    pub running: bool,
    pub current_item: Option<String>,
    pub completed_items: usize,
    pub total_items: usize,
}

#[derive(Debug, Serialize)]
pub struct RunSummary {
    pub searched: usize,
    pub grabbed: usize,
    pub skipped: usize,
    pub errors: usize,
    pub reconciled_missing: usize,
}

pub async fn status(
    State(state): State<AppState>,
) -> Result<Json<AutomationStatus>, (StatusCode, String)> {
    let enabled = setting_bool(&state, "automation.enabled", false).await?;
    let interval_minutes = setting_u64(&state, "automation.interval_minutes", 30).await?;
    let monitored_movies =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM movies WHERE monitored=1")
            .fetch_one(&state.db)
            .await
            .map_err(internal)?;
    let monitored_series =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM series WHERE monitored=1")
            .fetch_one(&state.db)
            .await
            .map_err(internal)?;
    let wanted_episodes = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM series_episodes e
        JOIN series s ON s.id=e.series_id
        WHERE s.monitored=1 AND e.monitored=1 AND e.has_file=0
          AND (e.air_date IS NULL OR e.air_date<=date('now','localtime'))
    "#,
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;
    let states=sqlx::query_as::<_,AutomationItemState>(
        "SELECT media_type,media_id,last_search_at,last_grab_at,last_grab_title,last_grab_score,last_error,status FROM automation_state ORDER BY COALESCE(last_search_at,'') DESC"
    ).fetch_all(&state.db).await.map_err(internal)?;

    let progress = state.automation_runtime.progress.read().await.clone();
    Ok(Json(AutomationStatus {
        enabled,
        interval_minutes,
        monitored_movies,
        monitored_series,
        wanted_episodes,
        states,
        running: state.automation_runtime.running.load(Ordering::SeqCst),
        current_item: progress.current_item,
        completed_items: progress.completed_items,
        total_items: progress.total_items,
    }))
}

pub async fn run_now(
    State(state): State<AppState>,
) -> Result<Json<RunSummary>, (StatusCode, String)> {
    let Some(_guard) = state.automation_runtime.begin() else {
        return Err((
            StatusCode::CONFLICT,
            "Automation is already running. Wait for the current cycle to finish.".into(),
        ));
    };
    let result = run_cycle(&state).await;
    Ok(Json(result))
}

pub fn spawn_scheduler(state: AppState) {
    tokio::spawn(async move {
        loop {
            let enabled = setting_bool(&state, "automation.enabled", false)
                .await
                .unwrap_or(false);
            let interval = setting_u64(&state, "automation.interval_minutes", 30)
                .await
                .unwrap_or(30)
                .clamp(5, 1440);
            if enabled {
                let Some(_guard) = state.automation_runtime.begin() else {
                    sleep(Duration::from_secs(interval * 60)).await;
                    continue;
                };
                let summary = run_cycle(&state).await;
                drop(_guard);
                tracing::info!(
                    searched = summary.searched,
                    grabbed = summary.grabbed,
                    skipped = summary.skipped,
                    errors = summary.errors,
                    "automation cycle completed"
                );
            }
            sleep(Duration::from_secs(interval * 60)).await;
        }
    });
}

pub(crate) async fn run_cycle(state: &AppState) -> RunSummary {
    let mut summary = RunSummary {
        searched: 0,
        grabbed: 0,
        skipped: 0,
        errors: 0,
        reconciled_missing: 0,
    };
    summary.reconciled_missing = importer::reconcile_missing_torrents(state)
        .await
        .unwrap_or(0);

    // Ordered least-recently-searched first (never-searched sorts first via the
    // empty COALESCE), not by id: with MAX_SEARCHES_PER_CYCLE capping how many
    // targets get a real search, plain id order would let whatever sorts last
    // starve permanently instead of eventually getting its turn.
    let movies = sqlx::query_as::<_, (i64, String)>(
        r#"SELECT m.id,m.title FROM movies m
           LEFT JOIN automation_state a ON a.media_type='movie' AND a.media_id=m.id
           WHERE m.monitored=1 ORDER BY COALESCE(a.last_search_at,''),m.id"#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let series_rows = sqlx::query_as::<_, (i64, String)>(
        r#"SELECT s.id,s.name FROM series s
           LEFT JOIN automation_state a ON a.media_type='series' AND a.media_id=s.id
           WHERE s.monitored=1 ORDER BY COALESCE(a.last_search_at,''),s.id"#,
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    state
        .automation_runtime
        .reset(movies.len() + series_rows.len())
        .await;
    // Alternates movies and series instead of exhausting the cycle on every
    // movie before a single series gets a turn: with more monitored movies
    // than MAX_SEARCHES_PER_CYCLE, series would otherwise never be searched.
    let mut movies_iter = movies.into_iter();
    let mut series_iter = series_rows.into_iter();
    loop {
        let movie_next = movies_iter.next();
        let series_next = series_iter.next();
        if movie_next.is_none() && series_next.is_none() {
            break;
        }
        if let Some((id, title)) = movie_next {
            state
                .automation_runtime
                .item(format!("Movie · {title}"))
                .await;
            process_movie(state, id, &title, &mut summary, true).await;
            state.automation_runtime.done().await;
        }
        if let Some((series_id, title)) = series_next {
            state
                .automation_runtime
                .item(format!("Series · {title}"))
                .await;
            process_series(state, series_id, &title, &mut summary, true).await;
            state.automation_runtime.done().await;
        }
    }
    summary
}

/// Runs automation for exactly one item. Native request clients use this after an
/// approval so adding one title never starts searches for the whole monitored library.
pub(crate) async fn run_media_cycle(
    state: &AppState,
    media_type: &str,
    media_id: i64,
) -> RunSummary {
    let mut summary = RunSummary {
        searched: 0,
        grabbed: 0,
        skipped: 0,
        errors: 0,
        reconciled_missing: 0,
    };
    summary.reconciled_missing = importer::reconcile_missing_torrents(state)
        .await
        .unwrap_or(0);
    match media_type {
        "movie" => {
            if let Ok(Some((id, title))) = sqlx::query_as::<_, (i64, String)>(
                "SELECT id,title FROM movies WHERE id=? AND monitored=1",
            )
            .bind(media_id)
            .fetch_optional(&state.db)
            .await
            {
                process_movie(state, id, &title, &mut summary, false).await;
            }
        }
        "series" => {
            if let Ok(Some((id, title))) = sqlx::query_as::<_, (i64, String)>(
                "SELECT id,name FROM series WHERE id=? AND monitored=1",
            )
            .bind(media_id)
            .fetch_optional(&state.db)
            .await
            {
                process_series(state, id, &title, &mut summary, false).await;
            }
        }
        _ => summary.errors += 1,
    }
    summary
}

async fn process_movie(
    state: &AppState,
    media_id: i64,
    title: &str,
    summary: &mut RunSummary,
    periodic: bool,
) {
    let spec = match search_api::resolve_media_spec(state, "movie", media_id).await {
        Ok(v) => v,
        Err((_, e)) => {
            record_error(state, "movie", media_id, &e).await;
            summary.errors += 1;
            return;
        }
    };
    let Some(profile_id) = spec.profile_id else {
        summary.skipped += 1;
        return;
    };
    let profile = match profiles::get_quality_profile_by_id(&state.db, profile_id).await {
        Ok(v) => v,
        Err((_, e)) => {
            record_error(state, "movie", media_id, &e).await;
            summary.errors += 1;
            return;
        }
    };
    if !profile.enabled {
        summary.skipped += 1;
        return;
    }

    let current_score=sqlx::query_scalar::<_,Option<i32>>(
        "SELECT MAX(quality_score) FROM media_files WHERE media_type='movie' AND media_id=? AND file_exists=1"
    ).bind(media_id).fetch_one(&state.db).await.unwrap_or(None);

    if let Some(score) = current_score
        && (!profile.upgrade_allowed || score >= profile.cutoff_score)
    {
        summary.skipped += 1;
        touch_search(state, "movie", media_id, "cutoff").await;
        return;
    }

    if summary.searched >= MAX_SEARCHES_PER_CYCLE {
        summary.skipped += 1;
        return;
    }
    // Unlike series, nothing upstream of this point checks for a job already in
    // flight: `current_score` comes from imported media_files, which stays empty
    // for the entire queued/downloading window, so without this a movie search
    // could fire — and grab a second release — every cycle until the first
    // finishes importing.
    if already_grabbed(state, "movie", media_id, None, None).await {
        summary.skipped += 1;
        return;
    }
    let response = match search_api::search_media_internal(state, &spec, periodic).await {
        Ok(v) => v,
        Err((_, e)) => {
            record_error(state, "movie", media_id, &e).await;
            summary.errors += 1;
            return;
        }
    };
    summary.searched += 1;
    let Some(best) = response.results.into_iter().find(|x| x.accepted) else {
        touch_search(state, "movie", media_id, "wanted").await;
        summary.skipped += 1;
        return;
    };

    if let Some(score) = current_score
        && best.match_score + best.profile_score <= score + 25
    {
        touch_search(state, "movie", media_id, "upgrade-wait").await;
        summary.skipped += 1;
        return;
    }

    // The search above took real time, during which RSS or another automation
    // run could have grabbed this same movie — the cheap check earlier only
    // ruled that out before searching. Re-check while holding the lock, right
    // before committing, so whichever path gets here first is the only one
    // that grabs.
    let _lock = state.grab_lock.lock().await;
    if already_grabbed(state, "movie", media_id, None, None).await {
        summary.skipped += 1;
        return;
    }

    let req = GrabRequest {
        indexer_id: best.indexer_id.clone(),
        indexer_name: Some(best.indexer_name.clone()),
        title: best.title.clone(),
        download_url: best.download_url.clone(),
        details_url: best.details_url.clone(),
        category: None,
        media_type: Some("movie".into()),
        media_id: Some(media_id),
        profile_id: Some(profile.id),
        season_number: None,
        episode_number: None,
        is_season_pack: Some(false),
    };

    match search_api::grab_internal(state, &req).await {
        Ok(()) => {
            let _=sqlx::query(r#"
                INSERT INTO automation_state(
                  media_type,media_id,last_search_at,last_grab_at,last_grab_title,last_grab_score,last_error,status
                )
                VALUES('movie',?,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP,?,?,NULL,'grabbed')
                ON CONFLICT(media_type,media_id) DO UPDATE SET
                  last_search_at=CURRENT_TIMESTAMP,last_grab_at=CURRENT_TIMESTAMP,
                  last_grab_title=excluded.last_grab_title,last_grab_score=excluded.last_grab_score,
                  last_error=NULL,status='grabbed'
            "#).bind(media_id).bind(&best.title).bind(best.score).execute(&state.db).await;
            history::record(
                &state.db,
                "automation.grabbed",
                title,
                Some(&format!(
                    "{} · score {} · {}",
                    profile.name, best.score, best.indexer_name
                )),
                "info",
            )
            .await;
            summary.grabbed += 1;
        }
        Err((_, e)) => {
            record_error(state, "movie", media_id, &e).await;
            summary.errors += 1;
        }
    }
}

async fn process_series(
    state: &AppState,
    series_id: i64,
    title: &str,
    summary: &mut RunSummary,
    periodic: bool,
) {
    let rows=sqlx::query_as::<_,(i32,i32,String,bool,Option<i32>)>(r#"
        SELECT e.season_number,e.episode_number,e.name,e.has_file,
               (SELECT MAX(mf.quality_score) FROM episode_files ef JOIN media_files mf ON mf.id=ef.media_file_id
                 WHERE ef.episode_id=e.id AND mf.file_exists=1) AS current_score
        FROM series_episodes e
        WHERE e.series_id=? AND e.monitored=1
          AND (e.air_date IS NULL OR e.air_date<=date('now','localtime'))
        ORDER BY e.season_number,e.episode_number
    "#).bind(series_id).fetch_all(&state.db).await.unwrap_or_default();

    // If the monitored aired library is completely empty, a "Complete Series" release is often
    // the best automatic target. Manual Series search already surfaced these, but older automation
    // only searched Sxx / SxxExx targets and therefore silently ignored titles without season tokens.
    let missing_total = rows
        .iter()
        .filter(|(_, _, _, has_file, _)| !*has_file)
        .count();
    let available_total = rows
        .iter()
        .filter(|(_, _, _, has_file, _)| *has_file)
        .count();
    if missing_total >= 2 && available_total == 0 && summary.searched < MAX_SEARCHES_PER_CYCLE {
        match try_complete_series(state, series_id, title, periodic).await {
            Ok(CompleteSeriesAttempt::Grabbed) => {
                summary.searched += 1;
                summary.grabbed += 1;
                return;
            }
            Ok(CompleteSeriesAttempt::SearchedNoMatch) => {
                summary.searched += 1;
            }
            Ok(CompleteSeriesAttempt::AlreadyActive) => {
                summary.skipped += missing_total;
                return;
            }
            Ok(CompleteSeriesAttempt::NotApplicable) => {}
            Err(e) => {
                record_error(state, "series", series_id, &e).await;
                summary.errors += 1;
                // Fall through to season/episode automation instead of blocking the series.
            }
        }
    }

    #[allow(clippy::type_complexity)]
    let mut by_season: BTreeMap<i32, Vec<(i32, String, bool, Option<i32>)>> = BTreeMap::new();
    for (season, episode, name, has_file, current_score) in rows {
        if season <= 0 {
            continue;
        }
        by_season
            .entry(season)
            .or_default()
            .push((episode, name, has_file, current_score));
    }

    for (season_number, episodes) in by_season {
        // Resolved with episode=None (season/series level), not the first
        // episode's own override: a pack covers the whole season, so its
        // profile must be a season-wide decision, not whatever the first
        // listed episode happens to be individually configured with.
        let pack_profile_id = match series::effective_profile_for_episode(
            state,
            series_id,
            season_number,
            None,
        )
        .await
        {
            Ok(v) => v,
            Err((_, error)) => {
                record_series_target_error(state, series_id, season_number, None, &error).await;
                summary.errors += 1;
                None
            }
        };
        let pack_profile = match pack_profile_id {
            Some(id) => profiles::get_quality_profile_by_id(&state.db, id)
                .await
                .ok(),
            None => None,
        };

        let missing = episodes
            .iter()
            .filter(|(_, _, has_file, _)| !*has_file)
            .count();
        // Whether the pack attempt (if any) means missing episodes are covered:
        // true when capped, when a pack job is already active, or when a pack
        // was just grabbed. False when a pack search ran but found nothing —
        // that falls through to per-episode search below instead of leaving
        // those episodes permanently unsearched (they'd otherwise never get a
        // turn as long as `series_prefer_pack` stays on and no pack exists).
        let mut pack_covers_missing = false;
        if let Some(pack_profile) = pack_profile.as_ref().filter(|p| p.enabled)
            && pack_profile.rules.series_prefer_pack
            && missing >= 2
        {
            if summary.searched >= MAX_SEARCHES_PER_CYCLE
                || has_active_series_job(state, series_id, season_number, None, true).await
            {
                summary.skipped += missing;
                pack_covers_missing = true;
            } else {
                pack_covers_missing = process_series_target(
                    state,
                    series_id,
                    title,
                    season_number,
                    None,
                    true,
                    pack_profile,
                    None,
                    summary,
                    periodic,
                )
                .await;
            }
        }

        for (episode_number, episode_name, has_file, current_score) in episodes {
            if !has_file && pack_covers_missing {
                continue;
            }
            // Resolved per episode, not reused from the season/pack lookup
            // above: a specific episode can carry its own profile override
            // (a different cutoff, a disabled profile, 4K/strict-language
            // requirements) that must govern that one target, not whatever
            // the pack-level or first-episode profile happened to be.
            let profile_id = match series::effective_profile_for_episode(
                state,
                series_id,
                season_number,
                Some(episode_number),
            )
            .await
            {
                Ok(v) => v,
                Err((_, error)) => {
                    record_series_target_error(
                        state,
                        series_id,
                        season_number,
                        Some(episode_number),
                        &error,
                    )
                    .await;
                    summary.errors += 1;
                    continue;
                }
            };
            let Some(profile_id) = profile_id else {
                summary.skipped += 1;
                continue;
            };
            let profile = match profiles::get_quality_profile_by_id(&state.db, profile_id).await {
                Ok(v) => v,
                Err((_, error)) => {
                    record_series_target_error(
                        state,
                        series_id,
                        season_number,
                        Some(episode_number),
                        &error,
                    )
                    .await;
                    summary.errors += 1;
                    continue;
                }
            };
            if !profile.enabled {
                summary.skipped += 1;
                continue;
            }
            if has_file
                && (!profile.upgrade_allowed || current_score.unwrap_or(0) >= profile.cutoff_score)
            {
                summary.skipped += 1;
                continue;
            }
            if summary.searched >= MAX_SEARCHES_PER_CYCLE {
                summary.skipped += 1;
                continue;
            }
            if has_active_series_job(state, series_id, season_number, Some(episode_number), false)
                .await
            {
                summary.skipped += 1;
                continue;
            }
            process_series_target(
                state,
                series_id,
                &format!("{title} · {episode_name}"),
                season_number,
                Some(episode_number),
                false,
                &profile,
                current_score,
                summary,
                periodic,
            )
            .await;
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn process_series_target(
    state: &AppState,
    series_id: i64,
    display_title: &str,
    season_number: i32,
    episode_number: Option<i32>,
    is_season_pack: bool,
    profile: &profiles::QualityProfile,
    current_score: Option<i32>,
    summary: &mut RunSummary,
    periodic: bool,
) -> bool {
    let spec = match search_api::resolve_series_target_spec(
        state,
        series_id,
        season_number,
        episode_number,
    )
    .await
    {
        Ok(v) => v,
        Err((_, e)) => {
            record_series_target_error(state, series_id, season_number, episode_number, &e).await;
            summary.errors += 1;
            return false;
        }
    };

    let response = match search_api::search_media_internal(state, &spec, periodic).await {
        Ok(v) => v,
        Err((_, e)) => {
            record_series_target_error(state, series_id, season_number, episode_number, &e).await;
            summary.errors += 1;
            return false;
        }
    };
    summary.searched += 1;

    let best = response
        .results
        .into_iter()
        .filter(|x| x.accepted)
        .find(|x| matches_series_target(&x.title, season_number, episode_number));

    let Some(best) = best else {
        touch_series_target(
            state,
            series_id,
            season_number,
            episode_number,
            "wanted",
            None,
            None,
        )
        .await;
        summary.skipped += 1;
        return false;
    };
    if let Some(score) = current_score
        && best.match_score + best.profile_score <= score + 25
    {
        touch_series_target(
            state,
            series_id,
            season_number,
            episode_number,
            "upgrade-wait",
            None,
            Some(score),
        )
        .await;
        summary.skipped += 1;
        return false;
    }

    // See process_movie's identical comment: the search above took real time,
    // during which RSS or a concurrent automation run could have grabbed this
    // exact season/episode. Re-check while holding the lock, right before
    // committing.
    let _lock = state.grab_lock.lock().await;
    if has_active_series_job(
        state,
        series_id,
        season_number,
        episode_number,
        is_season_pack,
    )
    .await
    {
        summary.skipped += 1;
        return false;
    }

    let req = GrabRequest {
        indexer_id: best.indexer_id.clone(),
        indexer_name: Some(best.indexer_name.clone()),
        title: best.title.clone(),
        download_url: best.download_url.clone(),
        details_url: best.details_url.clone(),
        category: None,
        media_type: Some("series".into()),
        media_id: Some(series_id),
        profile_id: Some(profile.id),
        season_number: Some(season_number),
        episode_number,
        is_season_pack: Some(is_season_pack),
    };

    match search_api::grab_internal(state, &req).await {
        Ok(()) => {
            touch_series_target(
                state,
                series_id,
                season_number,
                episode_number,
                "grabbed",
                Some(&best.title),
                Some(best.score),
            )
            .await;
            history::record(
                &state.db,
                if is_season_pack {
                    "automation.season_pack_grabbed"
                } else {
                    "automation.episode_grabbed"
                },
                display_title,
                Some(&format!(
                    "S{:02}{} · {} · score {} · {}",
                    season_number,
                    episode_number
                        .map(|e| format!("E{:02}", e))
                        .unwrap_or_default(),
                    profile.name,
                    best.score,
                    best.indexer_name
                )),
                "info",
            )
            .await;
            summary.grabbed += 1;
            true
        }
        Err((_, e)) => {
            record_series_target_error(state, series_id, season_number, episode_number, &e).await;
            summary.errors += 1;
            false
        }
    }
}

enum CompleteSeriesAttempt {
    Grabbed,
    SearchedNoMatch,
    AlreadyActive,
    NotApplicable,
}

async fn try_complete_series(
    state: &AppState,
    series_id: i64,
    title: &str,
    periodic: bool,
) -> Result<CompleteSeriesAttempt, String> {
    // Use the series-level profile. If the user has explicit season/episode profile overrides,
    // a single Complete Series torrent could violate those policies, so keep automation conservative.
    let profile_id =
        sqlx::query_scalar::<_, Option<i64>>("SELECT quality_profile_id FROM series WHERE id=?")
            .bind(series_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| e.to_string())?
            .flatten();
    let Some(profile_id) = profile_id else {
        return Ok(CompleteSeriesAttempt::NotApplicable);
    };

    let override_count=sqlx::query_scalar::<_,i64>(r#"
        SELECT
          (SELECT COUNT(*) FROM series_seasons WHERE series_id=? AND quality_profile_id IS NOT NULL)
          +
          (SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND quality_profile_id IS NOT NULL)
    "#).bind(series_id).bind(series_id).fetch_one(&state.db).await
        .map_err(|e|e.to_string())?;
    if override_count > 0 {
        return Ok(CompleteSeriesAttempt::NotApplicable);
    }

    let profile = profiles::get_quality_profile_by_id(&state.db, profile_id)
        .await
        .map_err(|e| e.1)?;
    if !profile.enabled || !profile.rules.series_accept_complete {
        return Ok(CompleteSeriesAttempt::NotApplicable);
    }

    // 'cleaned' excluded: the caller only reaches try_complete_series when
    // the series currently has zero available episodes and >=2 missing (see
    // process_series), so a historical "this was handled once" record must
    // not block repairing a season that has since regressed. Checking only
    // for another complete-series job (as this used to) missed that a
    // season-pack or per-episode job can just as easily already be covering
    // this series — a complete-series grab on top of that would be a
    // redundant, overlapping download of content already in flight.
    let active = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM download_jobs
        WHERE media_type='series' AND media_id=?
          AND status IN ('queued','downloading','completed','seeding')
    "#,
    )
    .bind(series_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);
    if active > 0 {
        return Ok(CompleteSeriesAttempt::AlreadyActive);
    }

    let spec = search_api::resolve_media_spec(state, "series", series_id)
        .await
        .map_err(|e| e.1)?;
    let response = search_api::search_media_internal(state, &spec, periodic)
        .await
        .map_err(|e| e.1)?;

    let best = response
        .results
        .into_iter()
        .filter(|x| x.accepted)
        .find(|x| is_complete_series_title(&x.title));

    let Some(best) = best else {
        touch_search(state, "series", series_id, "wanted-complete").await;
        return Ok(CompleteSeriesAttempt::SearchedNoMatch);
    };

    // See process_movie's identical comment: re-check while holding the lock,
    // right before committing, since the search above took real time.
    let _lock = state.grab_lock.lock().await;
    let still_active = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM download_jobs
        WHERE media_type='series' AND media_id=?
          AND status IN ('queued','downloading','completed','seeding')
    "#,
    )
    .bind(series_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);
    if still_active > 0 {
        return Ok(CompleteSeriesAttempt::AlreadyActive);
    }

    let req = GrabRequest {
        indexer_id: best.indexer_id.clone(),
        indexer_name: Some(best.indexer_name.clone()),
        title: best.title.clone(),
        download_url: best.download_url.clone(),
        details_url: best.details_url.clone(),
        category: None,
        media_type: Some("series".into()),
        media_id: Some(series_id),
        profile_id: Some(profile.id),
        season_number: None,
        episode_number: None,
        // Reuse the existing pack/import path. With no season_number this represents a full-series pack.
        is_season_pack: Some(true),
    };

    search_api::grab_internal(state, &req)
        .await
        .map_err(|e| e.1)?;

    let _=sqlx::query(r#"
        INSERT INTO automation_state(
          media_type,media_id,last_search_at,last_grab_at,last_grab_title,last_grab_score,last_error,status
        )
        VALUES('series',?,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP,?,?,NULL,'complete-series-grabbed')
        ON CONFLICT(media_type,media_id) DO UPDATE SET
          last_search_at=CURRENT_TIMESTAMP,last_grab_at=CURRENT_TIMESTAMP,
          last_grab_title=excluded.last_grab_title,last_grab_score=excluded.last_grab_score,
          last_error=NULL,status='complete-series-grabbed'
    "#).bind(series_id).bind(&best.title).bind(best.score).execute(&state.db).await;

    history::record(
        &state.db,
        "automation.complete_series_grabbed",
        title,
        Some(&format!(
            "{} · score {} · {}",
            profile.name, best.score, best.indexer_name
        )),
        "info",
    )
    .await;

    Ok(CompleteSeriesAttempt::Grabbed)
}

async fn has_active_series_job(
    state: &AppState,
    series_id: i64,
    season_number: i32,
    episode_number: Option<i32>,
    is_pack: bool,
) -> bool {
    // 'cleaned' does NOT count as active, for both branches: the caller only
    // reaches this check when the *current* missing-episode count for this
    // season is already known to be >=2 (process_series recomputes it fresh
    // every cycle from has_file), so a job history that says "this pack was
    // handled once" must not override what the library actually looks like
    // right now. A stale 'cleaned' record from before an episode was deleted,
    // or before a new one aired, would otherwise block ever repairing it.
    // Either branch also treats an active complete-series job as covering
    // this target, and the per-episode branch also treats an active
    // season-pack job for this season as covering it — a pack or a
    // complete-series download in flight already satisfies a pack/episode
    // request for the same content, so without this a scheduler/RSS/manual
    // path that doesn't share scope with another could start an overlapping
    // duplicate of something already being fetched.
    let count = if is_pack {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM download_jobs
            WHERE media_type='series' AND media_id=?
              AND status IN ('queued','downloading','completed','seeding')
              AND (
                (season_number=? AND is_season_pack=1)
                OR (season_number IS NULL AND episode_number IS NULL)
              )
        "#,
        )
        .bind(series_id)
        .bind(season_number)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0)
    } else {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM download_jobs
            WHERE media_type='series' AND media_id=?
              AND status IN ('queued','downloading','completed','seeding')
              AND (
                (season_number=? AND episode_number=?)
                OR (season_number=? AND episode_number IS NULL)
                OR (season_number IS NULL AND episode_number IS NULL)
              )
        "#,
        )
        .bind(series_id)
        .bind(season_number)
        .bind(episode_number)
        .bind(season_number)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0)
    };
    count > 0
}

async fn touch_series_target(
    state: &AppState,
    series_id: i64,
    season_number: i32,
    episode_number: Option<i32>,
    status: &str,
    title: Option<&str>,
    score: Option<i32>,
) {
    let ep = episode_number.unwrap_or(0);
    let grabbed = status == "grabbed";
    let _=sqlx::query(r#"
        INSERT INTO series_target_state(
          series_id,season_number,episode_number,last_search_at,last_grab_at,last_grab_title,last_grab_score,last_error,status
        ) VALUES(?,?,?,CURRENT_TIMESTAMP,CASE WHEN ? THEN CURRENT_TIMESTAMP ELSE NULL END,?,?,NULL,?)
        ON CONFLICT(series_id,season_number,episode_number) DO UPDATE SET
          last_search_at=CURRENT_TIMESTAMP,
          last_grab_at=CASE WHEN excluded.last_grab_at IS NOT NULL THEN excluded.last_grab_at ELSE series_target_state.last_grab_at END,
          last_grab_title=COALESCE(excluded.last_grab_title,series_target_state.last_grab_title),
          last_grab_score=COALESCE(excluded.last_grab_score,series_target_state.last_grab_score),
          last_error=NULL,status=excluded.status
    "#)
        .bind(series_id).bind(season_number).bind(ep).bind(grabbed)
        .bind(title).bind(score).bind(status)
        .execute(&state.db).await;
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
async fn setting_u64(
    state: &AppState,
    key: &str,
    default: u64,
) -> Result<u64, (StatusCode, String)> {
    Ok(settings::get_value(&state.db, key)
        .await
        .map_err(internal)?
        .and_then(|v| v.parse().ok())
        .unwrap_or(default))
}
fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{SeriesStrategy, is_complete_series_title, matches_series_target};

    #[test]
    fn complete_series_is_identified_before_fallbacks() {
        let strategy = if is_complete_series_title("Example.The.Complete.Series.1080p") {
            SeriesStrategy::Complete
        } else if matches_series_target("Example.S01.Season.Pack.1080p", 1, None) {
            SeriesStrategy::SeasonPack
        } else {
            SeriesStrategy::Episode
        };
        assert_eq!(strategy, SeriesStrategy::Complete);
    }

    #[test]
    fn falls_back_to_season_pack_when_complete_is_not_valid() {
        let strategy = if is_complete_series_title("Example.S01.Season.Pack.1080p") {
            SeriesStrategy::Complete
        } else if matches_series_target("Example.S01.Season.Pack.1080p", 1, None) {
            SeriesStrategy::SeasonPack
        } else {
            SeriesStrategy::Episode
        };
        assert_eq!(strategy, SeriesStrategy::SeasonPack);
    }

    #[test]
    fn falls_back_to_episode_without_matching_pack() {
        let strategy = if is_complete_series_title("Example.S01E02.1080p") {
            SeriesStrategy::Complete
        } else if matches_series_target("Example.S01E02.1080p", 1, None) {
            SeriesStrategy::SeasonPack
        } else {
            SeriesStrategy::Episode
        };
        assert_eq!(strategy, SeriesStrategy::Episode);
        assert!(matches_series_target("Example.S01E02.1080p", 1, Some(2)));
    }
}
