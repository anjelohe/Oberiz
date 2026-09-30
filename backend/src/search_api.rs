use crate::{
    AppState,
    cardigann::{self, ReleaseResult, SearchContext, SearchFailure},
    history, profiles, releases, series, tvdb,
};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ReleaseSearchQuery {
    pub query: String,
    pub media_type: Option<String>,
    pub media_id: Option<i64>,
    pub profile_id: Option<i64>,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub indexer_id: Option<String>,
    pub season_number: Option<i32>,
    pub episode_number: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct AggregateSearchResponse {
    pub status: &'static str,
    pub results: Vec<ReleaseResult>,
    pub failures: Vec<SearchFailure>,
    pub searched_indexers: usize,
    pub profile_id: Option<i64>,
    pub profile_name: Option<String>,
    pub cutoff_score: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct GrabRequest {
    pub indexer_id: String,
    pub indexer_name: Option<String>,
    pub title: String,
    pub download_url: Option<String>,
    pub details_url: Option<String>,
    pub category: Option<String>,
    pub media_type: Option<String>,
    pub media_id: Option<i64>,
    pub profile_id: Option<i64>,
    pub season_number: Option<i32>,
    pub episode_number: Option<i32>,
    pub is_season_pack: Option<bool>,
}

#[derive(Debug, Clone)]
pub(crate) struct MediaSearchSpec {
    pub media_type: String,
    pub query: String,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<i32>,
    pub target_season: Option<i32>,
    pub target_episode: Option<i32>,
    pub tmdb_id: Option<String>,
    pub imdb_id: Option<String>,
    pub profile_id: Option<i64>,
    pub indexer_id: Option<String>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(q): Query<ReleaseSearchQuery>,
) -> Result<Json<AggregateSearchResponse>, (StatusCode, String)> {
    if q.query.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "La búsqueda no puede estar vacía".into(),
        ));
    }
    let media_type = q.media_type.unwrap_or_else(|| "movie".into());
    let spec = resolve_spec(
        &state,
        &media_type,
        q.media_id,
        q.profile_id,
        &q.query,
        q.tmdb_id,
        q.imdb_id,
        q.indexer_id,
        q.season_number,
        q.episode_number,
    )
    .await?;
    let response = search_media_internal(&state, &spec).await?;
    history::record(
        &state.db,
        "releases.search",
        &spec.title,
        Some(&format!(
            "{} indexers · {} results · {} errors",
            response.searched_indexers,
            response.results.len(),
            response.failures.len()
        )),
        "info",
    )
    .await;
    Ok(Json(response))
}

pub(crate) async fn resolve_media_spec(
    state: &AppState,
    media_type: &str,
    media_id: i64,
) -> Result<MediaSearchSpec, (StatusCode, String)> {
    resolve_spec(
        state,
        media_type,
        Some(media_id),
        None,
        "",
        None,
        None,
        None,
        None,
        None,
    )
    .await
}

pub(crate) async fn resolve_series_target_spec(
    state: &AppState,
    series_id: i64,
    season_number: i32,
    episode_number: Option<i32>,
) -> Result<MediaSearchSpec, (StatusCode, String)> {
    let row = sqlx::query_as::<_, (String, Option<String>, Option<i32>, i64)>(
        "SELECT name,original_name,year,tmdb_id FROM series WHERE id=?",
    )
    .bind(series_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Serie no encontrada".into()))?;

    let profile_id =
        series::effective_profile_for_episode(state, series_id, season_number, episode_number)
            .await?;
    let query = match episode_number {
        Some(ep) => format!("{} S{:02}E{:02}", row.0, season_number, ep),
        None => format!("{} S{:02}", row.0, season_number),
    };
    Ok(MediaSearchSpec {
        media_type: "series".into(),
        query,
        title: row.0,
        original_title: row.1,
        year: row.2,
        target_season: Some(season_number),
        target_episode: episode_number,
        tmdb_id: Some(row.3.to_string()),
        imdb_id: None,
        profile_id,
        indexer_id: None,
    })
}

#[allow(clippy::too_many_arguments)]
async fn resolve_spec(
    state: &AppState,
    media_type: &str,
    media_id: Option<i64>,
    requested_profile: Option<i64>,
    fallback_query: &str,
    tmdb_id: Option<String>,
    imdb_id: Option<String>,
    indexer_id: Option<String>,
    target_season: Option<i32>,
    target_episode: Option<i32>,
) -> Result<MediaSearchSpec, (StatusCode, String)> {
    if media_type != "movie" && media_type != "series" {
        return Err((
            StatusCode::BAD_REQUEST,
            "media_type debe ser movie o series".into(),
        ));
    }

    if let Some(id) = media_id {
        if media_type == "movie" {
            let row=sqlx::query_as::<_,(String,Option<String>,Option<i32>,i64,Option<i64>)>("SELECT title,original_title,year,tmdb_id,quality_profile_id FROM movies WHERE id=?")
                .bind(id).fetch_optional(&state.db).await.map_err(internal)?
                .ok_or_else(||(StatusCode::NOT_FOUND,"Película no encontrada".into()))?;
            let query = if let Some(year) = row.2 {
                format!("{} {}", row.0, year)
            } else {
                row.0.clone()
            };
            return Ok(MediaSearchSpec {
                media_type: media_type.into(),
                query,
                title: row.0,
                original_title: row.1,
                year: row.2,
                target_season: None,
                target_episode: None,
                tmdb_id: Some(row.3.to_string()),
                imdb_id,
                profile_id: requested_profile.or(row.4),
                indexer_id,
            });
        }
        let row = sqlx::query_as::<_, (String, Option<String>, Option<i32>, i64, Option<i64>)>(
            "SELECT name,original_name,year,tmdb_id,quality_profile_id FROM series WHERE id=?",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Serie no encontrada".into()))?;
        let query = if let Some(year) = row.2 {
            format!("{} {}", row.0, year)
        } else {
            row.0.clone()
        };
        return Ok(MediaSearchSpec {
            media_type: media_type.into(),
            query,
            title: row.0,
            original_title: row.1,
            year: row.2,
            target_season,
            target_episode,
            tmdb_id: Some(row.3.to_string()),
            imdb_id,
            profile_id: requested_profile.or(row.4),
            indexer_id,
        });
    }

    Ok(MediaSearchSpec {
        media_type: media_type.into(),
        query: fallback_query.into(),
        title: fallback_query.into(),
        original_title: None,
        year: None,
        target_season,
        target_episode,
        tmdb_id,
        imdb_id,
        profile_id: requested_profile,
        indexer_id,
    })
}

pub(crate) async fn search_media_internal(
    state: &AppState,
    spec: &MediaSearchSpec,
) -> Result<AggregateSearchResponse, (StatusCode, String)> {
    let indexers: Vec<(String, String)> = if let Some(id) = &spec.indexer_id {
        let config = sqlx::query_scalar::<_, String>(
            "SELECT config_json FROM indexer_configs WHERE indexer_id=?",
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?
        .unwrap_or_else(|| "{}".into());
        vec![(id.clone(), config)]
    } else {
        sqlx::query_as::<_, (String, String)>(
            "SELECT indexer_id,config_json FROM indexer_configs WHERE enabled=1",
        )
        .fetch_all(&state.db)
        .await
        .map_err(internal)?
    };
    let mut prioritized = indexers
        .into_iter()
        .map(|(id, config)| {
            let priority = serde_json::from_str::<serde_json::Value>(&config)
                .ok()
                .and_then(|value| value.get("oberiz_priority").cloned())
                .and_then(|value| {
                    value
                        .as_i64()
                        .or_else(|| value.as_str().and_then(|value| value.parse::<i64>().ok()))
                })
                .unwrap_or(100);
            (id, priority)
        })
        .collect::<Vec<_>>();
    prioritized.sort_by(|(left_id, left), (right_id, right)| {
        left.cmp(right).then_with(|| left_id.cmp(right_id))
    });
    let ids = prioritized
        .iter()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let priorities = prioritized.into_iter().collect::<HashMap<_, _>>();

    let profile = match spec.profile_id {
        Some(id) => Some(profiles::get_quality_profile_by_id(&state.db, id).await?),
        None => None,
    };
    let language = if let Some(p) = &profile {
        profiles::language_for_profile(&state.db, p).await?
    } else {
        None
    };

    if ids.is_empty() {
        return Ok(AggregateSearchResponse {
            status: "ok",
            results: vec![],
            failures: vec![],
            searched_indexers: 0,
            profile_id: profile.as_ref().map(|p| p.id),
            profile_name: profile.as_ref().map(|p| p.name.clone()),
            cutoff_score: profile.as_ref().map(|p| p.cutoff_score),
        });
    }

    // Keep any season/year suffix of the displayed-title query when trying
    // TMDB's original title (for example "Lioness S03").
    let alternate_keywords = spec.original_title.as_deref().and_then(|original| {
        let original = original.trim();
        if original.is_empty() || original.eq_ignore_ascii_case(&spec.title) {
            return None;
        }
        let suffix = spec.query.strip_prefix(&spec.title).unwrap_or("");
        Some(format!("{original}{suffix}"))
    });
    let tvdb_id = if spec.media_type == "series" {
        tvdb::find_series_id(
            state,
            &spec.title,
            spec.original_title.as_deref(),
            spec.year,
        )
        .await
    } else {
        None
    };
    let ctx = SearchContext {
        keywords: spec.query.clone(),
        alternate_keywords,
        tmdb_id: spec.tmdb_id.clone(),
        imdb_id: spec.imdb_id.clone(),
        tvdb_id,
        media_type: spec.media_type.clone(),
        year: spec.year,
        season: spec.target_season,
        episode: spec.target_episode,
    };
    let mut results = Vec::new();
    let mut failures = Vec::new();

    for id in &ids {
        match cardigann::search_indexer(state, id, &ctx).await {
            Ok(mut rows) => {
                for row in &mut rows {
                    if let Some(profile) = &profile {
                        let eval = profiles::evaluate_release(
                            row,
                            profile,
                            language.as_ref(),
                            &spec.title,
                            spec.original_title.as_deref(),
                            spec.year,
                            &spec.media_type,
                            spec.target_season,
                        );
                        row.base_score = row.score;
                        row.profile_score = eval.profile_score;
                        row.match_score = eval.match_score;
                        row.score = eval.total_score;
                        row.accepted = eval.accepted;
                        row.reasons = eval.reasons;
                        row.rejection_reasons = eval.rejection_reasons;
                    }
                }
                results.append(&mut rows)
            }
            Err(error) => failures.push(SearchFailure {
                indexer_id: id.clone(),
                indexer_name: id.clone(),
                error,
            }),
        }
    }

    let prefer_indexer_priority = profile
        .as_ref()
        .is_some_and(|profile| profile.rules.prefer_indexer_priority);
    results.sort_by(|a, b| {
        let indexer_order = || {
            priorities
                .get(&a.indexer_id)
                .unwrap_or(&100)
                .cmp(priorities.get(&b.indexer_id).unwrap_or(&100))
        };
        let quality_order = || b.score.cmp(&a.score);

        b.accepted
            .cmp(&a.accepted)
            .then_with(|| {
                if prefer_indexer_priority {
                    indexer_order()
                } else {
                    quality_order()
                }
            })
            .then_with(|| {
                if prefer_indexer_priority {
                    quality_order()
                } else {
                    indexer_order()
                }
            })
            .then_with(|| b.seeders.unwrap_or(0).cmp(&a.seeders.unwrap_or(0)))
    });
    results.truncate(250);

    Ok(AggregateSearchResponse {
        status: "ok",
        results,
        failures,
        searched_indexers: ids.len(),
        profile_id: profile.as_ref().map(|p| p.id),
        profile_name: profile.as_ref().map(|p| p.name.clone()),
        cutoff_score: profile.as_ref().map(|p| p.cutoff_score),
    })
}

pub async fn grab(
    State(state): State<AppState>,
    Json(req): Json<GrabRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    grab_internal(&state, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn grab_internal(
    state: &AppState,
    req: &GrabRequest,
) -> Result<(), (StatusCode, String)> {
    let payload = cardigann::fetch_release_bytes_or_url(
        state,
        &req.indexer_id,
        req.download_url.as_deref(),
        req.details_url.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;

    let source = req
        .indexer_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(req.indexer_id.as_str());
    let media_type = req.media_type.as_deref().unwrap_or("movie");
    let media_id = req.media_id.unwrap_or(0);

    let profile = resolve_grab_profile(
        state,
        media_type,
        media_id,
        req.profile_id,
        req.season_number,
        req.episode_number,
    )
    .await?;
    let category = if let Some(profile) = profile.as_ref() {
        profile.qbittorrent_category.trim().to_string()
    } else {
        req.category.clone().unwrap_or_default().trim().to_string()
    };

    let parsed = releases::parse(&req.title);
    let (media_title, year) = media_identity(state, media_type, media_id, &req.title).await?;
    let tags_template = profile
        .as_ref()
        .map(|p| p.qbittorrent_tags_template.as_str())
        .unwrap_or("[tracker]");
    let tag_name = configured_tag_name(state, &req.indexer_id, source).await?;
    let visible_tags = render_qb_tags(
        tags_template,
        &tag_name,
        &req.indexer_id,
        profile.as_ref().map(|p| p.name.as_str()).unwrap_or(""),
        media_type,
        parsed.resolution.as_deref(),
        parsed.source.as_deref(),
        parsed.codec.as_deref(),
        parsed.language.as_deref(),
        &media_title,
        year,
    );

    let job_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO download_jobs(
          media_type,media_id,release_title,indexer_id,indexer_name,category,qbittorrent_tags,
          season_number,episode_number,is_season_pack,status
        )
        VALUES(?,?,?,?,?,?,?,?,?,?, 'queued')
        RETURNING id
    "#,
    )
    .bind(media_type)
    .bind(media_id)
    .bind(&req.title)
    .bind(&req.indexer_id)
    .bind(source)
    .bind(&category)
    .bind(&visible_tags)
    .bind(req.season_number)
    .bind(req.episode_number)
    .bind(req.is_season_pack.unwrap_or(false))
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;

    let result = match payload {
        cardigann::GrabPayload::Url(url) => {
            state
                .download_client
                .add_magnet(
                    state,
                    &url,
                    &category,
                    &req.title,
                    &visible_tags,
                    Some(job_id),
                )
                .await
        }
        cardigann::GrabPayload::Torrent(bytes) => {
            state
                .download_client
                .add_torrent_file(
                    state,
                    bytes,
                    &category,
                    &req.title,
                    &visible_tags,
                    Some(job_id),
                    None,
                    false,
                )
                .await
        }
    };

    let qb_hash = match result {
        Ok(hash) => hash,
        Err(error) => {
            let _=sqlx::query("UPDATE download_jobs SET status='error',last_error=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
                .bind(&error.1).bind(job_id).execute(&state.db).await;
            history::record(
                &state.db,
                "release.grab_failed",
                &req.title,
                Some(&error.1),
                "error",
            )
            .await;
            return Err(error);
        }
    };

    if let Err(error) =
        sqlx::query("UPDATE download_jobs SET qb_hash=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(&qb_hash)
            .bind(job_id)
            .execute(&state.db)
            .await
    {
        // qBittorrent can return an already-known hash when the same release is submitted
        // again. The unique hash index then rejects the new row; do not leave it in `queued`,
        // because that would block every later automated search for this media.
        let message = format!("Torrent already linked to another job: {error}");
        sqlx::query("UPDATE download_jobs SET status='duplicate',last_error=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(&message)
            .bind(job_id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        history::record(
            &state.db,
            "release.duplicate",
            &req.title,
            Some("qBittorrent already has this torrent; the existing job remains authoritative."),
            "warning",
        )
        .await;
        return Ok(());
    }

    history::record(
        &state.db,
        "release.grabbed",
        &req.title,
        Some(&format!(
            "{} -> qBittorrent · hash={} · category={} · tags={} · job {}",
            source, qb_hash, category, visible_tags, job_id
        )),
        "info",
    )
    .await;
    Ok(())
}

async fn configured_tag_name(
    state: &AppState,
    indexer_id: &str,
    fallback: &str,
) -> Result<String, (StatusCode, String)> {
    let custom = sqlx::query_scalar::<_, Option<String>>(
        "SELECT json_extract(config_json,'$.oberiz_tag_name') FROM indexer_configs WHERE indexer_id=?",
    )
    .bind(indexer_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .flatten()
    .unwrap_or_default();
    let custom = custom.trim();
    Ok(if custom.is_empty() {
        fallback.to_string()
    } else {
        custom.to_string()
    })
}

async fn resolve_grab_profile(
    state: &AppState,
    media_type: &str,
    media_id: i64,
    requested: Option<i64>,
    season_number: Option<i32>,
    episode_number: Option<i32>,
) -> Result<Option<profiles::QualityProfile>, (StatusCode, String)> {
    if let Some(id) = requested {
        return Ok(Some(
            profiles::get_quality_profile_by_id(&state.db, id).await?,
        ));
    }
    if media_id <= 0 {
        return Ok(None);
    }
    let id = if media_type == "series" {
        if let Some(season) = season_number {
            series::effective_profile_for_episode(state, media_id, season, episode_number).await?
        } else {
            sqlx::query_scalar::<_, Option<i64>>("SELECT quality_profile_id FROM series WHERE id=?")
                .bind(media_id)
                .fetch_optional(&state.db)
                .await
                .map_err(internal)?
                .flatten()
        }
    } else {
        sqlx::query_scalar::<_, Option<i64>>("SELECT quality_profile_id FROM movies WHERE id=?")
            .bind(media_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?
            .flatten()
    };
    match id {
        Some(id) => Ok(Some(
            profiles::get_quality_profile_by_id(&state.db, id).await?,
        )),
        None => Ok(None),
    }
}

async fn media_identity(
    state: &AppState,
    media_type: &str,
    media_id: i64,
    fallback: &str,
) -> Result<(String, Option<i32>), (StatusCode, String)> {
    if media_id <= 0 {
        return Ok((fallback.to_string(), None));
    }
    if media_type == "series" {
        Ok(
            sqlx::query_as::<_, (String, Option<i32>)>("SELECT name,year FROM series WHERE id=?")
                .bind(media_id)
                .fetch_optional(&state.db)
                .await
                .map_err(internal)?
                .unwrap_or_else(|| (fallback.to_string(), None)),
        )
    } else {
        Ok(
            sqlx::query_as::<_, (String, Option<i32>)>("SELECT title,year FROM movies WHERE id=?")
                .bind(media_id)
                .fetch_optional(&state.db)
                .await
                .map_err(internal)?
                .unwrap_or_else(|| (fallback.to_string(), None)),
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn render_qb_tags(
    template: &str,
    tracker: &str,
    indexer: &str,
    profile: &str,
    media_type: &str,
    resolution: Option<&str>,
    source: Option<&str>,
    codec: Option<&str>,
    language: Option<&str>,
    title: &str,
    year: Option<i32>,
) -> String {
    let replacements = [
        ("[tracker]", tracker),
        ("[indexer]", indexer),
        ("[profile]", profile),
        ("[media_type]", media_type),
        ("[resolution]", resolution.unwrap_or("")),
        ("[source]", source.unwrap_or("")),
        ("[codec]", codec.unwrap_or("")),
        ("[language]", language.unwrap_or("")),
        ("[title]", title),
    ];
    let mut rendered = template.to_string();
    for (token, value) in replacements {
        rendered = rendered.replace(token, value);
    }
    rendered = rendered.replace("[year]", &year.map(|y| y.to_string()).unwrap_or_default());
    let mut out = Vec::<String>::new();
    for raw in rendered.split(',') {
        let clean = raw.trim().replace(['\r', '\n'], " ");
        if !clean.is_empty() && !out.iter().any(|x| x.eq_ignore_ascii_case(&clean)) {
            out.push(clean);
        }
    }
    out.join(",")
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
