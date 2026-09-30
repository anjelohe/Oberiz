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
            return Ok(format!("skipped: {title} has no quality profile"));
        };
        let profile = profiles::get_quality_profile_by_id(&state.db, profile_id)
            .await
            .map_err(|e| e.1)?;
        if !profile.enabled {
            return Ok(format!("skipped: {title} profile disabled"));
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
        if !candidate.accepted {
            return Ok(format!(
                "rejected: {}",
                candidate.rejection_reasons.join(", ")
            ));
        }
        let current=sqlx::query_scalar::<_,Option<i32>>("SELECT MAX(quality_score) FROM media_files WHERE media_type='movie' AND media_id=? AND file_exists=1")
            .bind(id).fetch_one(&state.db).await.unwrap_or(None);
        if current.is_some_and(|score| !profile.upgrade_allowed || candidate.score <= score + 25) {
            return Ok("skipped: no quality upgrade".into());
        }
        if already_grabbed(state, "movie", id, None, None).await {
            return Ok("skipped: equivalent job already active".into());
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
        let Some(profile_id) = profile_id else {
            return Ok(format!("skipped: {title} has no quality profile"));
        };
        let profile = profiles::get_quality_profile_by_id(&state.db, profile_id)
            .await
            .map_err(|e| e.1)?;
        if !profile.enabled {
            return Ok(format!("skipped: {title} profile disabled"));
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
            None,
        );
        candidate.base_score = candidate.score;
        candidate.profile_score = evaluation.profile_score;
        candidate.match_score = evaluation.match_score;
        candidate.score = evaluation.total_score;
        candidate.accepted = evaluation.accepted;
        if !candidate.accepted {
            return Ok(format!(
                "rejected: {}",
                candidate.rejection_reasons.join(", ")
            ));
        }
        let missing=sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND monitored=1 AND has_file=0 AND (air_date IS NULL OR air_date<=date('now'))")
            .bind(id).fetch_one(&state.db).await.unwrap_or(0);
        if missing == 0 {
            return Ok("skipped: series is already complete".into());
        }
        if is_complete_series_title(&item.title) && profile.rules.series_accept_complete {
            if already_grabbed(state, "series", id, None, None).await {
                return Ok("skipped: equivalent job already active".into());
            }
            grab_rss(
                state,
                &candidate,
                "series",
                id,
                Some(profile.id),
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
            return Ok("skipped: series release has no season/episode target".into());
        };
        let season = caps
            .get(1)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0);
        let episode = caps.get(2).and_then(|m| m.as_str().parse::<i32>().ok());
        if season <= 0 {
            return Ok("skipped: invalid series target".into());
        }
        let is_pack = episode.is_none();
        if is_pack && !profile.rules.series_prefer_pack {
            return Ok("skipped: season packs disabled by profile".into());
        }
        let exists = if let Some(ep) = episode {
            sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND season_number=? AND episode_number=? AND monitored=1 AND has_file=0")
                .bind(id).bind(season).bind(ep).fetch_one(&state.db).await.unwrap_or(0)>0
        } else {
            sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND season_number=? AND monitored=1 AND has_file=0")
                .bind(id).bind(season).fetch_one(&state.db).await.unwrap_or(0)>0
        };
        if !exists {
            return Ok("skipped: target is not wanted".into());
        }
        if already_grabbed(state, "series", id, Some(season), episode).await {
            return Ok("skipped: equivalent job already active".into());
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
    Ok("ignored: no monitored match".into())
}

/// Whether this exact target has already been grabbed — actively downloading
/// right now, or successfully finished at some point in the past. `cleaned`
/// (seeding finished and the seed policy removed the torrent) counts too:
/// that job's files are already imported, so "cleaned up afterwards" must
/// not read as "never happened" and let automation grab it all over again.
async fn already_grabbed(
    state: &AppState,
    media_type: &str,
    media_id: i64,
    season: Option<i32>,
    episode: Option<i32>,
) -> bool {
    sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM download_jobs
        WHERE media_type=? AND media_id=? AND season_number IS ? AND episode_number IS ?
          AND status IN ('queued','downloading','completed','seeding','cleaned')"#,
    )
    .bind(media_type)
    .bind(media_id)
    .bind(season)
    .bind(episode)
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
    let normalized = title.to_lowercase();
    let season_token = format!("s{season_number:02}");
    if !normalized.contains(&season_token) {
        return false;
    }
    match episode_number {
        Some(episode) => normalized.contains(&format!("e{episode:02}")),
        None => {
            !regex::Regex::new(r"(?i)S\d{1,2}E\d{1,3}")
                .map(|re| re.is_match(&normalized))
                .unwrap_or(true)
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
          AND (e.air_date IS NULL OR e.air_date<=date('now'))
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
    if !state.automation_runtime.begin() {
        return Err((
            StatusCode::CONFLICT,
            "Automation is already running. Wait for the current cycle to finish.".into(),
        ));
    }
    let result = run_cycle(&state).await;
    state.automation_runtime.finish();
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
                if !state.automation_runtime.begin() {
                    sleep(Duration::from_secs(interval * 60)).await;
                    continue;
                }
                let summary = run_cycle(&state).await;
                state.automation_runtime.finish();
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

    let movies = sqlx::query_as::<_, (i64, String)>(
        "SELECT id,title FROM movies WHERE monitored=1 ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let series_rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT id,name FROM series WHERE monitored=1 ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    state
        .automation_runtime
        .reset(movies.len() + series_rows.len())
        .await;
    for (id, title) in movies {
        state
            .automation_runtime
            .item(format!("Movie · {title}"))
            .await;
        process_movie(state, id, &title, &mut summary, true).await;
        state.automation_runtime.done().await;
    }
    for (series_id, title) in series_rows {
        state
            .automation_runtime
            .item(format!("Series · {title}"))
            .await;
        process_series(state, series_id, &title, &mut summary, true).await;
        state.automation_runtime.done().await;
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
        && best.score <= score + 25
    {
        touch_search(state, "movie", media_id, "upgrade-wait").await;
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
          AND (e.air_date IS NULL OR e.air_date<=date('now'))
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
        let first_episode = episodes.first().map(|x| x.0);
        let profile_id = match series::effective_profile_for_episode(
            state,
            series_id,
            season_number,
            first_episode,
        )
        .await
        {
            Ok(v) => v,
            Err((_, error)) => {
                record_series_target_error(state, series_id, season_number, first_episode, &error)
                    .await;
                summary.errors += 1;
                continue;
            }
        };
        let Some(profile_id) = profile_id else {
            summary.skipped += episodes.len();
            continue;
        };
        let profile = match profiles::get_quality_profile_by_id(&state.db, profile_id).await {
            Ok(v) => v,
            Err((_, error)) => {
                record_series_target_error(state, series_id, season_number, first_episode, &error)
                    .await;
                summary.errors += 1;
                continue;
            }
        };
        if !profile.enabled {
            summary.skipped += episodes.len();
            continue;
        }

        let missing = episodes
            .iter()
            .filter(|(_, _, has_file, _)| !*has_file)
            .count();
        if profile.rules.series_prefer_pack && missing >= 2 {
            if summary.searched >= MAX_SEARCHES_PER_CYCLE
                || has_active_series_job(state, series_id, season_number, None, true).await
            {
                summary.skipped += missing;
            } else {
                process_series_target(
                    state,
                    series_id,
                    title,
                    season_number,
                    None,
                    true,
                    &profile,
                    None,
                    summary,
                    periodic,
                )
                .await;
            }
        }

        for (episode_number, episode_name, has_file, current_score) in episodes {
            if !has_file && profile.rules.series_prefer_pack && missing >= 2 {
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
) {
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
            return;
        }
    };

    let response = match search_api::search_media_internal(state, &spec, periodic).await {
        Ok(v) => v,
        Err((_, e)) => {
            record_series_target_error(state, series_id, season_number, episode_number, &e).await;
            summary.errors += 1;
            return;
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
        return;
    };
    if let Some(score) = current_score
        && best.score <= score + 25
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
        return;
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
        }
        Err((_, e)) => {
            record_series_target_error(state, series_id, season_number, episode_number, &e).await;
            summary.errors += 1;
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

    let active = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM download_jobs
        WHERE media_type='series' AND media_id=?
          AND season_number IS NULL AND episode_number IS NULL
          AND is_season_pack=1
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
    let count = if is_pack {
        sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM download_jobs
            WHERE media_type='series' AND media_id=? AND season_number=? AND is_season_pack=1
              AND status IN ('queued','downloading','completed','seeding')
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
            WHERE media_type='series' AND media_id=? AND season_number=? AND episode_number=?
              AND status IN ('queued','downloading','completed','seeding')
        "#,
        )
        .bind(series_id)
        .bind(season_number)
        .bind(episode_number)
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
