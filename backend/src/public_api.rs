use crate::{AppState, automation, profiles, series, settings, tmdb};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub media_type: String,
    pub tmdb_id: i64,
    pub quality_profile_id: Option<i64>,
    pub client_request_id: Option<String>,
    pub client_name: Option<String>,
    pub monitored: Option<bool>,
    pub monitor_mode: Option<String>,
    pub requested_seasons: Option<Vec<i32>>,
    pub monitor_future_seasons: Option<bool>,
    pub requested_by: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MediaRequestRow {
    pub id: i64,
    pub media_type: String,
    pub tmdb_id: i64,
    pub media_id: Option<i64>,
    pub quality_profile_id: Option<i64>,
    pub client_request_id: Option<String>,
    pub client_name: Option<String>,
    pub monitored: bool,
    pub monitor_mode: Option<String>,
    pub requested_seasons: Option<String>,
    pub monitor_future_seasons: bool,
    pub requested_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct MediaRequestView {
    #[serde(flatten)]
    pub request: MediaRequestRow,
    pub status: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicStatus {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Debug, Deserialize)]
struct TmdbMovieDetails {
    id: i64,
    title: String,
    #[serde(default)]
    original_title: String,
    #[serde(default)]
    overview: String,
    release_date: Option<String>,
    poster_path: Option<String>,
    backdrop_path: Option<String>,
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
}

pub async fn status(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PublicStatus>, (StatusCode, String)> {
    authorize(&state, &headers).await?;
    Ok(Json(PublicStatus {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn create_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateRequest>,
) -> Result<(StatusCode, Json<MediaRequestView>), (StatusCode, String)> {
    authorize(&state, &headers).await?;
    let view = create_request_internal(&state, payload).await?;
    Ok((StatusCode::CREATED, Json(view)))
}

/// Profiles are deliberately exposed through the authenticated public API so native
/// clients can offer Oberiz's real choices without depending on its internal UI API.
pub async fn list_quality_profiles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<profiles::QualityProfile>>, (StatusCode, String)> {
    authorize(&state, &headers).await?;
    let mut items = profiles::list_profiles_internal(&state.db, "movie").await?;
    items.extend(profiles::list_profiles_internal(&state.db, "series").await?);
    Ok(Json(
        items
            .into_iter()
            .filter(|profile| profile.enabled)
            .collect(),
    ))
}

pub(crate) async fn create_request_internal(
    state: &AppState,
    payload: CreateRequest,
) -> Result<MediaRequestView, (StatusCode, String)> {
    if !matches!(payload.media_type.as_str(), "movie" | "series") {
        return Err((
            StatusCode::BAD_REQUEST,
            "media_type debe ser movie o series".into(),
        ));
    }
    let client_name = payload
        .client_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let client_request_id = payload
        .client_request_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let (Some(client_name), Some(client_request_id)) = (client_name, client_request_id)
        && let Some((id,media_type,media_id))=sqlx::query_as::<_,(i64,String,Option<i64>)>("SELECT id,media_type,media_id FROM media_requests WHERE client_name=? AND client_request_id=? ORDER BY id DESC LIMIT 1")
            .bind(client_name).bind(client_request_id).fetch_optional(&state.db).await.map_err(internal)? {
            // A manual deletion of a movie/series must not make the client idempotency key permanent.
            // Discard only an orphaned request; healthy retries still return the original request.
            let exists=match media_id {
                None=>true,
                Some(media_id) if media_type=="movie"=>sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM movies WHERE id=?").bind(media_id).fetch_one(&state.db).await.map_err(internal)? > 0,
                Some(media_id)=>sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM series WHERE id=?").bind(media_id).fetch_one(&state.db).await.map_err(internal)? > 0,
            };
            // A Cinetta series can be extended later (for example T1 is already in the
            // library and the user asks for T2). Replace only its remote request so the
            // current season selection is applied again; movies and ordinary retries stay
            // fully idempotent.
            if exists && !(payload.media_type=="series" && payload.requested_seasons.is_some()) { return load_view(state,id).await; }
            sqlx::query("DELETE FROM media_requests WHERE id=?").bind(id).execute(&state.db).await.map_err(internal)?;
        }
    let monitored = payload.monitored.unwrap_or(true);
    let requested_seasons = payload
        .requested_seasons
        .as_ref()
        .map(|seasons| {
            seasons
                .iter()
                .copied()
                .filter(|season| *season > 0)
                .collect::<Vec<_>>()
        })
        .filter(|seasons| !seasons.is_empty());
    let monitor_future_seasons = payload.monitor_future_seasons.unwrap_or(false);
    let monitor_mode = if payload.media_type == "series" && requested_seasons.is_some() {
        if monitor_future_seasons {
            "future"
        } else {
            "none"
        }
    } else {
        payload.monitor_mode.as_deref().unwrap_or("all")
    };
    let media_id = if payload.media_type == "movie" {
        ensure_movie(
            state,
            payload.tmdb_id,
            payload.quality_profile_id,
            monitored,
        )
        .await?
    } else {
        ensure_series(
            state,
            payload.tmdb_id,
            payload.quality_profile_id,
            monitored,
            monitor_mode,
            requested_seasons.as_deref(),
            monitor_future_seasons,
        )
        .await?
    };
    let requested_seasons_json = requested_seasons
        .map(|seasons| serde_json::to_string(&seasons))
        .transpose()
        .map_err(internal)?;
    let id=sqlx::query_scalar::<_,i64>(r#"
        INSERT INTO media_requests(media_type,tmdb_id,media_id,quality_profile_id,client_request_id,client_name,monitored,monitor_mode,requested_seasons,monitor_future_seasons,requested_by)
        VALUES(?,?,?,?,?,?,?,?,?,?,?) RETURNING id
    "#).bind(&payload.media_type).bind(payload.tmdb_id).bind(media_id).bind(payload.quality_profile_id)
        .bind(client_request_id).bind(client_name).bind(monitored).bind(monitor_mode).bind(requested_seasons_json).bind(monitor_future_seasons).bind(&payload.requested_by)
        .fetch_one(&state.db).await.map_err(internal)?;

    let bg = state.clone();
    let media_type = payload.media_type.clone();
    tokio::spawn(async move {
        let _ = automation::run_media_cycle(&bg, &media_type, media_id).await;
    });
    load_view(state, id).await
}

pub async fn list_requests(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<MediaRequestView>>, (StatusCode, String)> {
    authorize(&state, &headers).await?;
    let rows=sqlx::query_as::<_,MediaRequestRow>(r#"
        SELECT id,media_type,tmdb_id,media_id,quality_profile_id,client_request_id,client_name,monitored,monitor_mode,requested_seasons,monitor_future_seasons,requested_by,created_at,updated_at
        FROM media_requests ORDER BY id DESC LIMIT 250
    "#).fetch_all(&state.db).await.map_err(internal)?;
    let mut out = Vec::new();
    for row in rows {
        out.push(to_view(&state, row).await?);
    }
    Ok(Json(out))
}

pub async fn get_request(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<Json<MediaRequestView>, (StatusCode, String)> {
    authorize(&state, &headers).await?;
    Ok(Json(load_view(&state, id).await?))
}

async fn load_view(state: &AppState, id: i64) -> Result<MediaRequestView, (StatusCode, String)> {
    let row=sqlx::query_as::<_,MediaRequestRow>(r#"
        SELECT id,media_type,tmdb_id,media_id,quality_profile_id,client_request_id,client_name,monitored,monitor_mode,requested_seasons,monitor_future_seasons,requested_by,created_at,updated_at
        FROM media_requests WHERE id=?
    "#).bind(id).fetch_optional(&state.db).await.map_err(internal)?
        .ok_or_else(||(StatusCode::NOT_FOUND,"Petición no encontrada".into()))?;
    to_view(state, row).await
}

async fn to_view(
    state: &AppState,
    row: MediaRequestRow,
) -> Result<MediaRequestView, (StatusCode, String)> {
    let Some(media_id) = row.media_id else {
        return Ok(MediaRequestView {
            request: row,
            status: "pending".into(),
            title: None,
        });
    };
    let active=sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM download_jobs WHERE media_type=? AND media_id=? AND status IN ('queued','downloading','completed','seeding')")
        .bind(&row.media_type).bind(media_id).fetch_one(&state.db).await.map_err(internal)?;
    if row.media_type == "movie" {
        let title = sqlx::query_scalar::<_, String>("SELECT title FROM movies WHERE id=?")
            .bind(media_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?;
        let available=sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM media_files WHERE media_type='movie' AND media_id=? AND file_exists=1")
            .bind(media_id).fetch_one(&state.db).await.map_err(internal)? > 0;
        let searched = automation_has_started(state, "movie", media_id).await?;
        let status = if available {
            "available"
        } else if active > 0 {
            "downloading"
        } else if searched {
            "searching"
        } else {
            "pending"
        };
        Ok(MediaRequestView {
            request: row,
            status: status.into(),
            title,
        })
    } else {
        let title = sqlx::query_scalar::<_, String>("SELECT name FROM series WHERE id=?")
            .bind(media_id)
            .fetch_optional(&state.db)
            .await
            .map_err(internal)?;
        let missing=sqlx::query_scalar::<_,i64>(r#"SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND monitored=1 AND has_file=0 AND (air_date IS NULL OR air_date<=date('now','localtime'))"#)
            .bind(media_id).fetch_one(&state.db).await.map_err(internal)?;
        let files = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM series_episodes WHERE series_id=? AND has_file=1",
        )
        .bind(media_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal)?;
        let searched = automation_has_started(state, "series", media_id).await?;
        let status = if active > 0 {
            "downloading"
        } else if missing > 0 && searched {
            "searching"
        } else if missing > 0 {
            "pending"
        } else if files > 0 {
            "available"
        } else {
            "monitoring"
        };
        Ok(MediaRequestView {
            request: row,
            status: status.into(),
            title,
        })
    }
}

async fn authorize(state: &AppState, headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    let enabled = settings::get_value(&state.db, "api.enabled")
        .await
        .map_err(internal)?
        .map(|v| v == "true")
        .unwrap_or(false);
    if !enabled {
        return Err((StatusCode::FORBIDDEN, "Public API is disabled".into()));
    }
    let expected_hash = settings::get_value(&state.db, "api.key_hash")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if expected_hash.is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "Public API key is not configured".into(),
        ));
    }
    let provided = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if provided.is_empty() || settings::hash_api_key(provided) != expected_hash {
        return Err((StatusCode::UNAUTHORIZED, "Invalid API key".into()));
    }
    Ok(())
}

async fn ensure_movie(
    state: &AppState,
    tmdb_id: i64,
    profile_id: Option<i64>,
    monitored: bool,
) -> Result<i64, (StatusCode, String)> {
    if let Some(id) = sqlx::query_scalar::<_, i64>("SELECT id FROM movies WHERE tmdb_id=?")
        .bind(tmdb_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?
    {
        sqlx::query("UPDATE movies SET monitored=?,quality_profile_id=COALESCE(?,quality_profile_id),updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(monitored).bind(profile_id).bind(id).execute(&state.db).await.map_err(internal)?;
        return Ok(id);
    }
    if let Some(id) = profile_id {
        let p = profiles::get_quality_profile_by_id(&state.db, id).await?;
        if p.media_type != "movie" {
            return Err((
                StatusCode::BAD_REQUEST,
                "Quality Profile is not for movies".into(),
            ));
        }
    }
    let credential = settings::get_value(&state.db, "tmdb.api_key")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if credential.is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "TMDB no está configurado".into(),
        ));
    }
    let response = tmdb::apply_tmdb_auth(
        state
            .http
            .get(format!("https://api.themoviedb.org/3/movie/{tmdb_id}"))
            .query(&[("language", "es-ES")]),
        &credential,
    )
    .send()
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("TMDB returned {}", response.status()),
        ));
    }
    let item = response
        .json::<TmdbMovieDetails>()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let year = item
        .release_date
        .as_deref()
        .and_then(|x| x.get(0..4))
        .and_then(|x| x.parse::<i32>().ok());
    let profile_id = match profile_id {
        Some(v) => Some(v),
        None => profiles::default_profile_id(&state.db, "movie")
            .await
            .map_err(internal)?,
    };
    sqlx::query_scalar::<_,i64>(r#"
        INSERT INTO movies(tmdb_id,title,original_title,year,overview,poster_path,backdrop_path,monitored,quality_profile_id)
        VALUES(?,?,?,?,?,?,?,?,?) RETURNING id
    "#).bind(item.id).bind(item.title).bind(item.original_title).bind(year).bind(item.overview)
        .bind(item.poster_path.map(|p|format!("https://image.tmdb.org/t/p/w500{p}")))
        .bind(item.backdrop_path.map(|p|format!("https://image.tmdb.org/t/p/w1280{p}")))
        .bind(monitored).bind(profile_id).fetch_one(&state.db).await.map_err(internal)
}

async fn ensure_series(
    state: &AppState,
    tmdb_id: i64,
    profile_id: Option<i64>,
    monitored: bool,
    monitor_mode: &str,
    requested_seasons: Option<&[i32]>,
    monitor_future_seasons: bool,
) -> Result<i64, (StatusCode, String)> {
    if let Some(id) = sqlx::query_scalar::<_, i64>("SELECT id FROM series WHERE tmdb_id=?")
        .bind(tmdb_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal)?
    {
        sqlx::query("UPDATE series SET monitored=?,quality_profile_id=COALESCE(?,quality_profile_id),monitor_mode=?,updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(monitored).bind(profile_id).bind(monitor_mode).bind(id).execute(&state.db).await.map_err(internal)?;
        series::refresh_series_metadata_internal(state, id).await?;
        if let Some(seasons) = requested_seasons {
            apply_requested_seasons(state, id, seasons, monitor_future_seasons).await?;
        }
        return Ok(id);
    }
    if let Some(id) = profile_id {
        let p = profiles::get_quality_profile_by_id(&state.db, id).await?;
        if p.media_type != "series" {
            return Err((
                StatusCode::BAD_REQUEST,
                "Quality Profile is not for series".into(),
            ));
        }
    }
    let credential = settings::get_value(&state.db, "tmdb.api_key")
        .await
        .map_err(internal)?
        .unwrap_or_default();
    if credential.is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "TMDB no está configurado".into(),
        ));
    }
    let response = tmdb::apply_tmdb_auth(
        state
            .http
            .get(format!("https://api.themoviedb.org/3/tv/{tmdb_id}"))
            .query(&[("language", "es-ES")]),
        &credential,
    )
    .send()
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("TMDB returned {}", response.status()),
        ));
    }
    let item = response
        .json::<TmdbSeriesDetails>()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    let year = item
        .first_air_date
        .as_deref()
        .and_then(|x| x.get(0..4))
        .and_then(|x| x.parse::<i32>().ok());
    let profile_id = match profile_id {
        Some(v) => Some(v),
        None => profiles::default_profile_id(&state.db, "series")
            .await
            .map_err(internal)?,
    };
    let id=sqlx::query_scalar::<_,i64>(r#"
        INSERT INTO series(tmdb_id,name,original_name,year,overview,poster_path,backdrop_path,monitored,quality_profile_id,monitor_mode)
        VALUES(?,?,?,?,?,?,?,?,?,?) RETURNING id
    "#).bind(item.id).bind(item.name).bind(item.original_name).bind(year).bind(item.overview)
        .bind(item.poster_path.map(|p|format!("https://image.tmdb.org/t/p/w500{p}")))
        .bind(item.backdrop_path.map(|p|format!("https://image.tmdb.org/t/p/w1280{p}")))
        .bind(monitored).bind(profile_id).bind(monitor_mode).fetch_one(&state.db).await.map_err(internal)?;
    series::refresh_series_metadata_internal(state, id).await?;
    if let Some(seasons) = requested_seasons {
        apply_requested_seasons(state, id, seasons, monitor_future_seasons).await?;
    }
    Ok(id)
}

async fn automation_has_started(
    state: &AppState,
    media_type: &str,
    media_id: i64,
) -> Result<bool, (StatusCode, String)> {
    Ok(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM automation_state WHERE media_type=? AND media_id=? AND last_search_at IS NOT NULL")
        .bind(media_type).bind(media_id).fetch_one(&state.db).await.map_err(internal)? > 0)
}

async fn apply_requested_seasons(
    state: &AppState,
    series_id: i64,
    seasons: &[i32],
    monitor_future: bool,
) -> Result<(), (StatusCode, String)> {
    let mut tx = state.db.begin().await.map_err(internal)?;
    sqlx::query("UPDATE series_seasons SET monitored=0,monitor_override=0,updated_at=CURRENT_TIMESTAMP WHERE series_id=? AND season_number>0")
        .bind(series_id).execute(&mut *tx).await.map_err(internal)?;
    sqlx::query("UPDATE series_episodes SET monitored=0,monitor_override=0,updated_at=CURRENT_TIMESTAMP WHERE series_id=? AND season_number>0")
        .bind(series_id).execute(&mut *tx).await.map_err(internal)?;
    for season in seasons {
        sqlx::query("UPDATE series_seasons SET monitored=1,monitor_override=1,updated_at=CURRENT_TIMESTAMP WHERE series_id=? AND season_number=?")
            .bind(series_id).bind(season).execute(&mut *tx).await.map_err(internal)?;
        sqlx::query("UPDATE series_episodes SET monitored=1,monitor_override=1,updated_at=CURRENT_TIMESTAMP WHERE series_id=? AND season_number=?")
            .bind(series_id).bind(season).execute(&mut *tx).await.map_err(internal)?;
    }
    if monitor_future {
        sqlx::query("UPDATE series_seasons SET monitor_override=NULL WHERE series_id=? AND season_number>0 AND monitor_override=0")
            .bind(series_id).execute(&mut *tx).await.map_err(internal)?;
        sqlx::query("UPDATE series_episodes SET monitor_override=NULL WHERE series_id=? AND season_number>0 AND monitor_override=0 AND air_date IS NOT NULL AND air_date>=date('now','localtime')")
            .bind(series_id).execute(&mut *tx).await.map_err(internal)?;
    }
    tx.commit().await.map_err(internal)?;
    if monitor_future {
        series::apply_monitor_mode_internal(state, series_id, "future").await?;
    }
    Ok(())
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
