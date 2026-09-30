use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, automation, history, profiles, settings, tmdb};

#[derive(Debug, Deserialize)]
pub struct CreateSeriesRequest {
    pub tmdb_id: i64,
    pub monitored: Option<bool>,
    pub library_path: Option<String>,
    pub quality_profile_id: Option<i64>,
    pub monitor_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSeriesRequest {
    pub monitored: Option<bool>,
    pub library_path: Option<String>,
    pub quality_profile_id: Option<i64>,
    pub monitor_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSeasonRequest {
    pub monitored: Option<bool>,
    pub quality_profile_id: Option<i64>,
    #[serde(default)]
    pub clear_quality_profile: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEpisodeRequest {
    pub monitored: Option<bool>,
    pub quality_profile_id: Option<i64>,
    #[serde(default)]
    pub clear_quality_profile: bool,
}

#[derive(Debug, Deserialize)]
struct TmdbSeasonSummary {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    overview: String,
    air_date: Option<String>,
    poster_path: Option<String>,
    season_number: i32,
    #[serde(default)]
    episode_count: i32,
}

#[derive(Debug, Deserialize)]
struct TmdbSeriesDetails {
    id: i64,
    name: String,
    #[serde(default)]
    original_name: String,
    #[serde(default)]
    overview: String,
    first_air_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
    #[serde(default)]
    seasons: Vec<TmdbSeasonSummary>,
}

#[derive(Debug, Deserialize)]
struct TmdbEpisode {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    overview: String,
    air_date: Option<String>,
    still_path: Option<String>,
    #[serde(default)]
    episode_number: i32,
    #[serde(default)]
    season_number: i32,
    runtime: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct TmdbSeasonDetails {
    id: i64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    overview: String,
    air_date: Option<String>,
    poster_path: Option<String>,
    season_number: i32,
    #[serde(default)]
    episodes: Vec<TmdbEpisode>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Series {
    pub id: i64,
    pub tmdb_id: i64,
    pub name: String,
    pub original_name: Option<String>,
    pub year: Option<i32>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub monitored: bool,
    pub library_path: Option<String>,
    pub quality_profile_id: Option<i64>,
    pub quality_profile_name: Option<String>,
    pub monitor_mode: String,
    pub metadata_synced_at: Option<String>,
    pub season_count: i64,
    pub episode_count: i64,
    pub monitored_episode_count: i64,
    pub available_episode_count: i64,
    pub missing_episode_count: i64,
    pub future_episode_count: i64,
    pub latest_season_number: Option<i32>,
    pub next_missing_season: Option<i32>,
    pub next_missing_episode: Option<i32>,
    pub next_missing_name: Option<String>,
    pub next_upcoming_season: Option<i32>,
    pub next_upcoming_episode: Option<i32>,
    pub next_upcoming_name: Option<String>,
    pub next_upcoming_air_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SeriesSeason {
    pub id: i64,
    pub series_id: i64,
    pub tmdb_season_id: Option<i64>,
    pub season_number: i32,
    pub name: String,
    pub overview: Option<String>,
    pub air_date: Option<String>,
    pub poster_path: Option<String>,
    pub episode_count: i32,
    pub monitored: bool,
    pub quality_profile_id: Option<i64>,
    pub quality_profile_name: Option<String>,
    pub effective_quality_profile_id: Option<i64>,
    pub effective_quality_profile_name: Option<String>,
    pub available_episodes: i64,
    pub monitored_episodes: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct SeriesEpisode {
    pub id: i64,
    pub series_id: i64,
    pub season_id: i64,
    pub tmdb_episode_id: Option<i64>,
    pub season_number: i32,
    pub episode_number: i32,
    pub name: String,
    pub overview: Option<String>,
    pub air_date: Option<String>,
    pub still_path: Option<String>,
    pub runtime: Option<i32>,
    pub monitored: bool,
    pub has_file: bool,
    pub quality_profile_id: Option<i64>,
    pub quality_profile_name: Option<String>,
    pub effective_quality_profile_id: Option<i64>,
    pub effective_quality_profile_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SeasonWithEpisodes {
    #[serde(flatten)]
    pub season: SeriesSeason,
    pub episodes: Vec<SeriesEpisode>,
}

#[derive(Debug, Serialize)]
pub struct SeriesDetailResponse {
    pub series: Series,
    pub seasons: Vec<SeasonWithEpisodes>,
}

async fn find_series(db: &sqlx::SqlitePool, id: i64) -> Result<Option<Series>, sqlx::Error> {
    sqlx::query_as::<_,Series>(r#"
        SELECT s.id,s.tmdb_id,s.name,s.original_name,s.year,s.overview,
               s.poster_path,s.backdrop_path,s.monitored,s.library_path,
               s.quality_profile_id,q.name AS quality_profile_name,
               s.monitor_mode,s.metadata_synced_at,
               (SELECT COUNT(*) FROM series_seasons ss WHERE ss.series_id=s.id AND ss.season_number>0) AS season_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0) AS episode_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1) AS monitored_episode_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.has_file=1) AS available_episode_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND (e.air_date IS NULL OR e.air_date<=date('now'))) AS missing_episode_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.air_date>date('now')) AS future_episode_count,
               (SELECT MAX(ss.season_number) FROM series_seasons ss WHERE ss.series_id=s.id AND ss.season_number>0) AS latest_season_number,
               (SELECT e.season_number FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND (e.air_date IS NULL OR e.air_date<=date('now')) ORDER BY e.season_number,e.episode_number LIMIT 1) AS next_missing_season,
               (SELECT e.episode_number FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND (e.air_date IS NULL OR e.air_date<=date('now')) ORDER BY e.season_number,e.episode_number LIMIT 1) AS next_missing_episode,
               (SELECT e.name FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND (e.air_date IS NULL OR e.air_date<=date('now')) ORDER BY e.season_number,e.episode_number LIMIT 1) AS next_missing_name,
               (SELECT e.season_number FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND e.air_date>date('now') ORDER BY e.air_date,e.season_number,e.episode_number LIMIT 1) AS next_upcoming_season,
               (SELECT e.episode_number FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND e.air_date>date('now') ORDER BY e.air_date,e.season_number,e.episode_number LIMIT 1) AS next_upcoming_episode,
               (SELECT e.name FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND e.air_date>date('now') ORDER BY e.air_date,e.season_number,e.episode_number LIMIT 1) AS next_upcoming_name,
               (SELECT e.air_date FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND e.air_date>date('now') ORDER BY e.air_date,e.season_number,e.episode_number LIMIT 1) AS next_upcoming_air_date,
               s.created_at,s.updated_at
        FROM series s
        LEFT JOIN quality_profiles q ON q.id=s.quality_profile_id
        WHERE s.id=?
    "#).bind(id).fetch_optional(db).await
}

fn validate_monitor_mode(value: &str) -> Result<&str, (StatusCode, String)> {
    match value {
        "all" | "future" | "missing" | "existing" | "first" | "latest" | "none" => Ok(value),
        _ => Err((
            StatusCode::BAD_REQUEST,
            "monitor_mode debe ser all, future, missing, existing, first, latest o none".into(),
        )),
    }
}

async fn tmdb_credential(state: &AppState) -> Result<String, (StatusCode, String)> {
    let value = settings::get_value(&state.db, "tmdb.api_key")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if value.trim().is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "TMDB no está configurado".into(),
        ));
    }
    Ok(value)
}

async fn fetch_tmdb_series(
    state: &AppState,
    tmdb_id: i64,
) -> Result<TmdbSeriesDetails, (StatusCode, String)> {
    let credential = tmdb_credential(state).await?;
    let response = tmdb::apply_tmdb_auth(
        state
            .http
            .get(format!("https://api.themoviedb.org/3/tv/{tmdb_id}"))
            .query(&[("language", "es-ES")]),
        &credential,
    )
    .send()
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Error conectando con TMDB: {e}"),
        )
    })?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err((StatusCode::NOT_FOUND, "La serie no existe en TMDB".into()));
    }
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("TMDB devolvió {}", response.status()),
        ));
    }
    response.json::<TmdbSeriesDetails>().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Respuesta TMDB no válida: {e}"),
        )
    })
}

async fn fetch_tmdb_season(
    state: &AppState,
    tmdb_id: i64,
    season_number: i32,
) -> Result<TmdbSeasonDetails, (StatusCode, String)> {
    let credential = tmdb_credential(state).await?;
    let response = tmdb::apply_tmdb_auth(
        state
            .http
            .get(format!(
                "https://api.themoviedb.org/3/tv/{tmdb_id}/season/{season_number}"
            ))
            .query(&[("language", "es-ES")]),
        &credential,
    )
    .send()
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Error conectando con TMDB: {e}"),
        )
    })?;
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("TMDB season {season_number} devolvió {}", response.status()),
        ));
    }
    response.json::<TmdbSeasonDetails>().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Respuesta de temporada TMDB no válida: {e}"),
        )
    })
}

pub async fn create_series(
    State(state): State<AppState>,
    Json(payload): Json<CreateSeriesRequest>,
) -> Result<(StatusCode, Json<Series>), (StatusCode, String)> {
    let item = fetch_tmdb_series(&state, payload.tmdb_id).await?;
    let year = item
        .first_air_date
        .as_deref()
        .and_then(|d| d.get(0..4))
        .and_then(|y| y.parse::<i32>().ok());
    let poster = item
        .poster_path
        .as_deref()
        .map(|p| format!("https://image.tmdb.org/t/p/w500{p}"));
    let backdrop = item
        .backdrop_path
        .as_deref()
        .map(|p| format!("https://image.tmdb.org/t/p/w1280{p}"));
    let quality_profile_id = match payload.quality_profile_id {
        Some(id) => {
            let p = profiles::get_quality_profile_by_id(&state.db, id).await?;
            if p.media_type != "series" {
                return Err((StatusCode::BAD_REQUEST, "El perfil no es de series".into()));
            }
            Some(id)
        }
        None => profiles::default_profile_id(&state.db, "series")
            .await
            .map_err(internal)?,
    };
    let monitor_mode =
        validate_monitor_mode(payload.monitor_mode.as_deref().unwrap_or("all"))?.to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO series(
          tmdb_id,name,original_name,year,overview,poster_path,backdrop_path,
          monitored,library_path,quality_profile_id,monitor_mode
        ) VALUES(?,?,?,?,?,?,?,?,?,?,?)
    "#,
    )
    .bind(item.id)
    .bind(&item.name)
    .bind(&item.original_name)
    .bind(year)
    .bind(&item.overview)
    .bind(&poster)
    .bind(&backdrop)
    .bind(payload.monitored.unwrap_or(true))
    .bind(&payload.library_path)
    .bind(quality_profile_id)
    .bind(&monitor_mode)
    .execute(&state.db)
    .await;

    let result = match result {
        Ok(v) => v,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err((StatusCode::CONFLICT, "La serie ya existe en Oberiz".into()));
        }
        Err(e) => return Err(internal(e)),
    };
    let id = result.last_insert_rowid();

    if let Err((_, error)) = sync_series_metadata(&state, id, Some(item)).await {
        history::record(
            &state.db,
            "series.metadata_error",
            &id.to_string(),
            Some(&error),
            "error",
        )
        .await;
    }

    let created = find_series(&state.db, id)
        .await
        .map_err(internal)?
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No se pudo leer la serie creada".into(),
            )
        })?;
    history::record(
        &state.db,
        "series.added",
        &created.name,
        Some("Serie añadida desde TMDB con temporadas/episodios"),
        "info",
    )
    .await;

    // One-off search for whatever the tracker already has, mirroring Radarr's
    // "search on add" — RSS only ever catches releases published after this
    // point, so anything already sitting on an indexer needs this once.
    let bg = state.clone();
    tokio::spawn(async move {
        let _ = automation::run_media_cycle(&bg, "series", id).await;
    });

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn list_series(
    State(state): State<AppState>,
) -> Result<Json<Vec<Series>>, (StatusCode, String)> {
    let items=sqlx::query_as::<_,Series>(r#"
        SELECT s.id,s.tmdb_id,s.name,s.original_name,s.year,s.overview,
               s.poster_path,s.backdrop_path,s.monitored,s.library_path,
               s.quality_profile_id,q.name AS quality_profile_name,
               s.monitor_mode,s.metadata_synced_at,
               (SELECT COUNT(*) FROM series_seasons ss WHERE ss.series_id=s.id AND ss.season_number>0) AS season_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0) AS episode_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1) AS monitored_episode_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.has_file=1) AS available_episode_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND (e.air_date IS NULL OR e.air_date<=date('now'))) AS missing_episode_count,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.air_date>date('now')) AS future_episode_count,
               (SELECT MAX(ss.season_number) FROM series_seasons ss WHERE ss.series_id=s.id AND ss.season_number>0) AS latest_season_number,
               (SELECT e.season_number FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND (e.air_date IS NULL OR e.air_date<=date('now')) ORDER BY e.season_number,e.episode_number LIMIT 1) AS next_missing_season,
               (SELECT e.episode_number FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND (e.air_date IS NULL OR e.air_date<=date('now')) ORDER BY e.season_number,e.episode_number LIMIT 1) AS next_missing_episode,
               (SELECT e.name FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND (e.air_date IS NULL OR e.air_date<=date('now')) ORDER BY e.season_number,e.episode_number LIMIT 1) AS next_missing_name,
               (SELECT e.season_number FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND e.air_date>date('now') ORDER BY e.air_date,e.season_number,e.episode_number LIMIT 1) AS next_upcoming_season,
               (SELECT e.episode_number FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND e.air_date>date('now') ORDER BY e.air_date,e.season_number,e.episode_number LIMIT 1) AS next_upcoming_episode,
               (SELECT e.name FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND e.air_date>date('now') ORDER BY e.air_date,e.season_number,e.episode_number LIMIT 1) AS next_upcoming_name,
               (SELECT e.air_date FROM series_episodes e WHERE e.series_id=s.id AND e.season_number>0 AND e.monitored=1 AND e.has_file=0 AND e.air_date>date('now') ORDER BY e.air_date,e.season_number,e.episode_number LIMIT 1) AS next_upcoming_air_date,
               s.created_at,s.updated_at
        FROM series s LEFT JOIN quality_profiles q ON q.id=s.quality_profile_id
        ORDER BY s.created_at DESC
    "#).fetch_all(&state.db).await.map_err(internal)?;
    Ok(Json(items))
}

pub async fn get_series(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SeriesDetailResponse>, (StatusCode, String)> {
    let series = find_series(&state.db, id)
        .await
        .map_err(internal)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Serie no encontrada".into()))?;
    Ok(Json(SeriesDetailResponse {
        series,
        seasons: load_seasons(&state, id).await?,
    }))
}

pub async fn refresh_series(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SeriesDetailResponse>, (StatusCode, String)> {
    if find_series(&state.db, id)
        .await
        .map_err(internal)?
        .is_none()
    {
        return Err((StatusCode::NOT_FOUND, "Serie no encontrada".into()));
    }
    sync_series_metadata(&state, id, None).await?;
    let series = find_series(&state.db, id).await.map_err(internal)?.unwrap();
    history::record(
        &state.db,
        "series.refreshed",
        &series.name,
        Some("Temporadas y episodios actualizados desde TMDB"),
        "info",
    )
    .await;
    Ok(Json(SeriesDetailResponse {
        series,
        seasons: load_seasons(&state, id).await?,
    }))
}

pub async fn list_seasons(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<SeasonWithEpisodes>>, (StatusCode, String)> {
    if find_series(&state.db, id)
        .await
        .map_err(internal)?
        .is_none()
    {
        return Err((StatusCode::NOT_FOUND, "Serie no encontrada".into()));
    }
    Ok(Json(load_seasons(&state, id).await?))
}

pub async fn update_series(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateSeriesRequest>,
) -> Result<Json<Series>, (StatusCode, String)> {
    if find_series(&state.db, id)
        .await
        .map_err(internal)?
        .is_none()
    {
        return Err((StatusCode::NOT_FOUND, "Serie no encontrada".into()));
    }

    if let Some(monitored) = payload.monitored {
        sqlx::query("UPDATE series SET monitored=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(monitored)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
    }
    if let Some(path) = payload.library_path {
        sqlx::query("UPDATE series SET library_path=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(path)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
    }
    if let Some(profile_id) = payload.quality_profile_id {
        let profile = profiles::get_quality_profile_by_id(&state.db, profile_id).await?;
        if profile.media_type != "series" {
            return Err((StatusCode::BAD_REQUEST, "El perfil no es de series".into()));
        }
        sqlx::query(
            "UPDATE series SET quality_profile_id=?,updated_at=CURRENT_TIMESTAMP WHERE id=?",
        )
        .bind(profile_id)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    }
    if let Some(mode) = payload.monitor_mode {
        let mode = validate_monitor_mode(&mode)?.to_string();
        sqlx::query("UPDATE series SET monitor_mode=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(&mode)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(internal)?;
        apply_monitor_mode_internal(&state, id, &mode).await?;
    }

    Ok(Json(
        find_series(&state.db, id)
            .await
            .map_err(internal)?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Serie no encontrada".into()))?,
    ))
}

pub async fn update_season(
    State(state): State<AppState>,
    Path((series_id, season_number)): Path<(i64, i32)>,
    Json(payload): Json<UpdateSeasonRequest>,
) -> Result<Json<SeasonWithEpisodes>, (StatusCode, String)> {
    let season_id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM series_seasons WHERE series_id=? AND season_number=?",
    )
    .bind(series_id)
    .bind(season_number)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Temporada no encontrada".into()))?;

    if let Some(monitored) = payload.monitored {
        let mut tx = state.db.begin().await.map_err(internal)?;
        sqlx::query("UPDATE series_seasons SET monitored=?,monitor_override=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(monitored).bind(monitored).bind(season_id).execute(&mut *tx).await.map_err(internal)?;
        sqlx::query("UPDATE series_episodes SET monitored=?,monitor_override=?,updated_at=CURRENT_TIMESTAMP WHERE season_id=?")
            .bind(monitored).bind(monitored).bind(season_id).execute(&mut *tx).await.map_err(internal)?;
        tx.commit().await.map_err(internal)?;
    }

    if payload.clear_quality_profile {
        sqlx::query("UPDATE series_seasons SET quality_profile_id=NULL,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(season_id).execute(&state.db).await.map_err(internal)?;
    } else if let Some(profile_id) = payload.quality_profile_id {
        validate_series_profile(&state, profile_id).await?;
        sqlx::query("UPDATE series_seasons SET quality_profile_id=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(profile_id).bind(season_id).execute(&state.db).await.map_err(internal)?;
    }

    let seasons = load_seasons(&state, series_id).await?;
    seasons
        .into_iter()
        .find(|x| x.season.season_number == season_number)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Temporada no encontrada".into()))
}

pub async fn update_episode(
    State(state): State<AppState>,
    Path((series_id, episode_id)): Path<(i64, i64)>,
    Json(payload): Json<UpdateEpisodeRequest>,
) -> Result<Json<SeriesEpisode>, (StatusCode, String)> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM series_episodes WHERE id=? AND series_id=?",
    )
    .bind(episode_id)
    .bind(series_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal)?;
    if exists == 0 {
        return Err((StatusCode::NOT_FOUND, "Episodio no encontrado".into()));
    }

    if let Some(monitored) = payload.monitored {
        sqlx::query("UPDATE series_episodes SET monitored=?,monitor_override=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(monitored).bind(monitored).bind(episode_id).execute(&state.db).await.map_err(internal)?;
    }
    if payload.clear_quality_profile {
        sqlx::query("UPDATE series_episodes SET quality_profile_id=NULL,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(episode_id).execute(&state.db).await.map_err(internal)?;
    } else if let Some(profile_id) = payload.quality_profile_id {
        validate_series_profile(&state, profile_id).await?;
        sqlx::query("UPDATE series_episodes SET quality_profile_id=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(profile_id).bind(episode_id).execute(&state.db).await.map_err(internal)?;
    }
    let rows = load_episode_rows(&state, series_id).await?;
    rows.into_iter()
        .find(|x| x.id == episode_id)
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Episodio no encontrado".into()))
}

pub async fn delete_series(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let current = find_series(&state.db, id).await.map_err(internal)?;
    let Some(current) = current else {
        return Err((StatusCode::NOT_FOUND, "Serie no encontrada".into()));
    };
    sqlx::query("DELETE FROM series WHERE id=?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal)?;
    history::record(
        &state.db,
        "series.deleted",
        &current.name,
        Some("Serie eliminada"),
        "info",
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn effective_profile_for_episode(
    state: &AppState,
    series_id: i64,
    season_number: i32,
    episode_number: Option<i32>,
) -> Result<Option<i64>, (StatusCode, String)> {
    if let Some(ep) = episode_number {
        let profile=sqlx::query_scalar::<_,Option<i64>>(
            "SELECT quality_profile_id FROM series_episodes WHERE series_id=? AND season_number=? AND episode_number=?"
        ).bind(series_id).bind(season_number).bind(ep).fetch_optional(&state.db).await.map_err(internal)?.flatten();
        if profile.is_some() {
            return Ok(profile);
        }
    }
    let season_profile = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT quality_profile_id FROM series_seasons WHERE series_id=? AND season_number=?",
    )
    .bind(series_id)
    .bind(season_number)
    .fetch_optional(&state.db)
    .await
    .map_err(internal)?
    .flatten();
    if season_profile.is_some() {
        return Ok(season_profile);
    }
    Ok(
        sqlx::query_scalar::<_, Option<i64>>("SELECT quality_profile_id FROM series WHERE id=?")
            .bind(series_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?
            .flatten(),
    )
}

async fn validate_series_profile(
    state: &AppState,
    profile_id: i64,
) -> Result<(), (StatusCode, String)> {
    let p = profiles::get_quality_profile_by_id(&state.db, profile_id).await?;
    if p.media_type != "series" {
        return Err((
            StatusCode::BAD_REQUEST,
            "El perfil seleccionado no es de series".into(),
        ));
    }
    Ok(())
}

pub(crate) async fn refresh_series_metadata_internal(
    state: &AppState,
    series_id: i64,
) -> Result<(), (StatusCode, String)> {
    sync_series_metadata(state, series_id, None).await
}

async fn sync_series_metadata(
    state: &AppState,
    series_id: i64,
    prefetched: Option<TmdbSeriesDetails>,
) -> Result<(), (StatusCode, String)> {
    let stored = find_series(&state.db, series_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Serie no encontrada".into()))?;
    let details = match prefetched {
        Some(v) => v,
        None => fetch_tmdb_series(state, stored.tmdb_id).await?,
    };

    let year = details
        .first_air_date
        .as_deref()
        .and_then(|d| d.get(0..4))
        .and_then(|y| y.parse::<i32>().ok());
    let poster = details
        .poster_path
        .as_deref()
        .map(|p| format!("https://image.tmdb.org/t/p/w500{p}"));
    let backdrop = details
        .backdrop_path
        .as_deref()
        .map(|p| format!("https://image.tmdb.org/t/p/w1280{p}"));

    sqlx::query(
        r#"
        UPDATE series SET name=?,original_name=?,year=?,overview=?,poster_path=?,backdrop_path=?,
          metadata_synced_at=CURRENT_TIMESTAMP,updated_at=CURRENT_TIMESTAMP WHERE id=?
    "#,
    )
    .bind(&details.name)
    .bind(&details.original_name)
    .bind(year)
    .bind(&details.overview)
    .bind(poster)
    .bind(backdrop)
    .bind(series_id)
    .execute(&state.db)
    .await
    .map_err(internal)?;

    for summary in details.seasons {
        let season = fetch_tmdb_season(state, stored.tmdb_id, summary.season_number)
            .await
            .unwrap_or(TmdbSeasonDetails {
                id: summary.id,
                name: summary.name.clone(),
                overview: summary.overview.clone(),
                air_date: summary.air_date.clone(),
                poster_path: summary.poster_path.clone(),
                season_number: summary.season_number,
                episodes: vec![],
            });
        let poster = season
            .poster_path
            .as_deref()
            .map(|p| format!("https://image.tmdb.org/t/p/w500{p}"));
        let season_id=sqlx::query_scalar::<_,i64>(r#"
            INSERT INTO series_seasons(
              series_id,tmdb_season_id,season_number,name,overview,air_date,poster_path,episode_count,monitored
            ) VALUES(?,?,?,?,?,?,?,?,?)
            ON CONFLICT(series_id,season_number) DO UPDATE SET
              tmdb_season_id=excluded.tmdb_season_id,name=excluded.name,overview=excluded.overview,
              air_date=excluded.air_date,poster_path=excluded.poster_path,episode_count=excluded.episode_count,
              updated_at=CURRENT_TIMESTAMP
            RETURNING id
        "#).bind(series_id).bind(season.id).bind(season.season_number)
            .bind(if season.name.trim().is_empty(){format!("Season {}",season.season_number)}else{season.name.clone()})
            .bind(&season.overview).bind(&season.air_date).bind(poster)
            .bind(if season.episodes.is_empty(){summary.episode_count}else{season.episodes.len() as i32})
            .bind(season.season_number!=0)
            .fetch_one(&state.db).await.map_err(internal)?;

        for ep in season.episodes {
            let still = ep
                .still_path
                .as_deref()
                .map(|p| format!("https://image.tmdb.org/t/p/w500{p}"));
            sqlx::query(r#"
                INSERT INTO series_episodes(
                  series_id,season_id,tmdb_episode_id,season_number,episode_number,name,overview,air_date,still_path,runtime,monitored
                ) VALUES(?,?,?,?,?,?,?,?,?,?,1)
                ON CONFLICT(series_id,season_number,episode_number) DO UPDATE SET
                  season_id=excluded.season_id,tmdb_episode_id=excluded.tmdb_episode_id,name=excluded.name,
                  overview=excluded.overview,air_date=excluded.air_date,still_path=excluded.still_path,
                  runtime=excluded.runtime,updated_at=CURRENT_TIMESTAMP
            "#).bind(series_id).bind(season_id).bind(ep.id).bind(ep.season_number).bind(ep.episode_number)
                .bind(if ep.name.trim().is_empty(){format!("Episode {}",ep.episode_number)}else{ep.name})
                .bind(ep.overview).bind(ep.air_date).bind(still).bind(ep.runtime)
                .execute(&state.db).await.map_err(internal)?;
        }
    }

    apply_monitor_mode_internal(state, series_id, &stored.monitor_mode).await?;
    Ok(())
}

pub(crate) async fn apply_monitor_mode_internal(
    state: &AppState,
    series_id: i64,
    mode: &str,
) -> Result<(), (StatusCode, String)> {
    let today = chrono_like_today();
    sqlx::query("UPDATE series_seasons SET monitored=0,updated_at=CURRENT_TIMESTAMP WHERE series_id=? AND monitor_override IS NULL")
        .bind(series_id).execute(&state.db).await.map_err(internal)?;
    sqlx::query("UPDATE series_episodes SET monitored=0,updated_at=CURRENT_TIMESTAMP WHERE series_id=? AND monitor_override IS NULL")
        .bind(series_id).execute(&state.db).await.map_err(internal)?;

    match mode {
        "none" => {}
        "first" => {
            sqlx::query("UPDATE series_seasons SET monitored=1 WHERE series_id=? AND season_number=1 AND monitor_override IS NULL")
                .bind(series_id).execute(&state.db).await.map_err(internal)?;
            sqlx::query("UPDATE series_episodes SET monitored=1 WHERE series_id=? AND season_number=1 AND monitor_override IS NULL")
                .bind(series_id).execute(&state.db).await.map_err(internal)?;
        }
        "latest" => {
            let latest=sqlx::query_scalar::<_,Option<i32>>(
                "SELECT MAX(season_number) FROM series_seasons WHERE series_id=? AND season_number>0"
            ).bind(series_id).fetch_one(&state.db).await.map_err(internal)?;
            if let Some(latest) = latest {
                sqlx::query("UPDATE series_seasons SET monitored=1 WHERE series_id=? AND season_number=? AND monitor_override IS NULL")
                    .bind(series_id).bind(latest).execute(&state.db).await.map_err(internal)?;
                sqlx::query("UPDATE series_episodes SET monitored=1 WHERE series_id=? AND season_number=? AND monitor_override IS NULL")
                    .bind(series_id).bind(latest).execute(&state.db).await.map_err(internal)?;
            }
        }
        "future" => {
            sqlx::query("UPDATE series_episodes SET monitored=1 WHERE series_id=? AND season_number>0 AND monitor_override IS NULL AND monitor_override IS NULL AND air_date IS NOT NULL AND air_date>=?")
                .bind(series_id).bind(&today).execute(&state.db).await.map_err(internal)?;
            sqlx::query("UPDATE series_seasons SET monitored=1 WHERE monitor_override IS NULL AND id IN (SELECT DISTINCT season_id FROM series_episodes WHERE series_id=? AND monitored=1)")
                .bind(series_id).execute(&state.db).await.map_err(internal)?;
        }
        "missing" => {
            sqlx::query("UPDATE series_episodes SET monitored=1 WHERE series_id=? AND season_number>0 AND monitor_override IS NULL AND monitor_override IS NULL AND has_file=0 AND (air_date IS NULL OR air_date<=?)")
                .bind(series_id).bind(&today).execute(&state.db).await.map_err(internal)?;
            sqlx::query("UPDATE series_seasons SET monitored=1 WHERE monitor_override IS NULL AND id IN (SELECT DISTINCT season_id FROM series_episodes WHERE series_id=? AND monitored=1)")
                .bind(series_id).execute(&state.db).await.map_err(internal)?;
        }
        "existing" => {
            sqlx::query("UPDATE series_episodes SET monitored=1 WHERE series_id=? AND season_number>0 AND monitor_override IS NULL AND monitor_override IS NULL AND has_file=1")
                .bind(series_id).execute(&state.db).await.map_err(internal)?;
            sqlx::query("UPDATE series_seasons SET monitored=1 WHERE monitor_override IS NULL AND id IN (SELECT DISTINCT season_id FROM series_episodes WHERE series_id=? AND monitored=1)")
                .bind(series_id).execute(&state.db).await.map_err(internal)?;
        }
        _ => {
            sqlx::query("UPDATE series_seasons SET monitored=1 WHERE series_id=? AND season_number>0 AND monitor_override IS NULL")
                .bind(series_id).execute(&state.db).await.map_err(internal)?;
            sqlx::query("UPDATE series_episodes SET monitored=1 WHERE series_id=? AND season_number>0 AND monitor_override IS NULL")
                .bind(series_id).execute(&state.db).await.map_err(internal)?;
        }
    }
    Ok(())
}

async fn load_seasons(
    state: &AppState,
    series_id: i64,
) -> Result<Vec<SeasonWithEpisodes>, (StatusCode, String)> {
    let seasons=sqlx::query_as::<_,SeriesSeason>(r#"
        SELECT ss.id,ss.series_id,ss.tmdb_season_id,ss.season_number,ss.name,ss.overview,ss.air_date,ss.poster_path,
               ss.episode_count,ss.monitored,ss.quality_profile_id,qp.name AS quality_profile_name,
               COALESCE(ss.quality_profile_id,s.quality_profile_id) AS effective_quality_profile_id,
               COALESCE(qp.name,sqp.name) AS effective_quality_profile_name,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.season_id=ss.id AND e.has_file=1) AS available_episodes,
               (SELECT COUNT(*) FROM series_episodes e WHERE e.season_id=ss.id AND e.monitored=1) AS monitored_episodes,
               ss.created_at,ss.updated_at
        FROM series_seasons ss
        JOIN series s ON s.id=ss.series_id
        LEFT JOIN quality_profiles qp ON qp.id=ss.quality_profile_id
        LEFT JOIN quality_profiles sqp ON sqp.id=s.quality_profile_id
        WHERE ss.series_id=?
        ORDER BY ss.season_number
    "#).bind(series_id).fetch_all(&state.db).await.map_err(internal)?;

    let episodes = load_episode_rows(state, series_id).await?;
    let mut out = Vec::new();
    for season in seasons {
        let rows = episodes
            .iter()
            .filter(|e| e.season_id == season.id)
            .cloned()
            .collect();
        out.push(SeasonWithEpisodes {
            season,
            episodes: rows,
        });
    }
    Ok(out)
}

async fn load_episode_rows(
    state: &AppState,
    series_id: i64,
) -> Result<Vec<SeriesEpisode>, (StatusCode, String)> {
    sqlx::query_as::<_,SeriesEpisode>(r#"
        SELECT e.id,e.series_id,e.season_id,e.tmdb_episode_id,e.season_number,e.episode_number,e.name,e.overview,
               e.air_date,e.still_path,e.runtime,e.monitored,e.has_file,e.quality_profile_id,
               ep.name AS quality_profile_name,
               COALESCE(e.quality_profile_id,ss.quality_profile_id,s.quality_profile_id) AS effective_quality_profile_id,
               COALESCE(ep.name,sp.name,basep.name) AS effective_quality_profile_name,
               e.created_at,e.updated_at
        FROM series_episodes e
        JOIN series_seasons ss ON ss.id=e.season_id
        JOIN series s ON s.id=e.series_id
        LEFT JOIN quality_profiles ep ON ep.id=e.quality_profile_id
        LEFT JOIN quality_profiles sp ON sp.id=ss.quality_profile_id
        LEFT JOIN quality_profiles basep ON basep.id=s.quality_profile_id
        WHERE e.series_id=?
        ORDER BY e.season_number,e.episode_number
    "#).bind(series_id).fetch_all(&state.db).await.map_err(internal)
}

fn chrono_like_today() -> String {
    // UTC calendar date without adding another dependency.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = now / 86400;
    civil_from_days(days as i64)
}

fn civil_from_days(days: i64) -> String {
    // Howard Hinnant civil_from_days, epoch 1970-01-01.
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    y += if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}")
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
