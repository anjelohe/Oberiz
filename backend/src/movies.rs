use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, automation, history, profiles, tmdb};

#[derive(Debug, Deserialize)]
pub struct CreateMovieRequest {
    pub tmdb_id: i64,
    pub monitored: Option<bool>,
    pub library_path: Option<String>,
    pub quality_profile_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMovieRequest {
    pub monitored: Option<bool>,
    pub library_path: Option<String>,
    pub quality_profile_id: Option<i64>,
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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Movie {
    pub id: i64,
    pub tmdb_id: i64,
    pub title: String,
    pub original_title: Option<String>,
    pub year: Option<i32>,
    pub overview: Option<String>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub monitored: bool,
    pub library_path: Option<String>,
    pub quality_profile_id: Option<i64>,
    pub quality_profile_name: Option<String>,
    pub file_count: i64,
    pub available: bool,
    pub current_resolution: Option<String>,
    pub current_quality_score: Option<i32>,
    pub upgrade_wanted: bool,
    pub created_at: String,
    pub updated_at: String,
}

async fn find_movie(db: &sqlx::SqlitePool, id: i64) -> Result<Option<Movie>, sqlx::Error> {
    sqlx::query_as::<_, Movie>(
        r#"
        SELECT m.id, m.tmdb_id, m.title, m.original_title, m.year, m.overview,
               m.poster_path, m.backdrop_path, m.monitored, m.library_path,
               m.quality_profile_id, q.name AS quality_profile_name,
               (SELECT COUNT(*) FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1) AS file_count,
               EXISTS(SELECT 1 FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1) AS available,
               (SELECT mf.resolution FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1 ORDER BY mf.quality_score DESC,mf.id DESC LIMIT 1) AS current_resolution,
               (SELECT MAX(mf.quality_score) FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1) AS current_quality_score,
               CASE WHEN q.upgrade_allowed=1
                         AND EXISTS(SELECT 1 FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1)
                         AND COALESCE((SELECT MAX(mf.quality_score) FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1),0) < q.cutoff_score
                    THEN 1 ELSE 0 END AS upgrade_wanted,
               m.created_at, m.updated_at
        FROM movies m
        LEFT JOIN quality_profiles q ON q.id=m.quality_profile_id
        WHERE m.id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn create_movie(
    State(state): State<AppState>,
    Json(payload): Json<CreateMovieRequest>,
) -> Result<(StatusCode, Json<Movie>), (StatusCode, String)> {
    let credential = crate::settings::get_value(&state.db, "tmdb.api_key")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .unwrap_or_default();

    if credential.is_empty() {
        return Err((
            StatusCode::PRECONDITION_REQUIRED,
            "TMDB no está configurado".into(),
        ));
    }

    let request = state
        .http
        .get(format!(
            "https://api.themoviedb.org/3/movie/{}",
            payload.tmdb_id
        ))
        .query(&[("language", "es-ES")]);

    let response = tmdb::apply_tmdb_auth(request, &credential)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Error conectando con TMDB: {e}"),
            )
        })?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err((
            StatusCode::NOT_FOUND,
            "La película no existe en TMDB".into(),
        ));
    }
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err((
            StatusCode::UNAUTHORIZED,
            "La credencial de TMDB no es válida".into(),
        ));
    }
    if !response.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("TMDB devolvió el estado HTTP {}", response.status()),
        ));
    }

    let movie = response.json::<TmdbMovieDetails>().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            format!("Respuesta TMDB no válida: {e}"),
        )
    })?;

    let year = movie
        .release_date
        .as_deref()
        .and_then(|d| d.get(0..4))
        .and_then(|y| y.parse::<i32>().ok());

    let poster_path = movie
        .poster_path
        .map(|p| format!("https://image.tmdb.org/t/p/w500{p}"));
    let backdrop_path = movie
        .backdrop_path
        .map(|p| format!("https://image.tmdb.org/t/p/w1280{p}"));

    let quality_profile_id = match payload.quality_profile_id {
        Some(id) => Some(id),
        None => profiles::default_profile_id(&state.db, "movie")
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?,
    };

    let result = sqlx::query(
        r#"
        INSERT INTO movies (
            tmdb_id, title, original_title, year, overview,
            poster_path, backdrop_path, monitored, library_path, quality_profile_id
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(movie.id)
    .bind(&movie.title)
    .bind(&movie.original_title)
    .bind(year)
    .bind(&movie.overview)
    .bind(&poster_path)
    .bind(&backdrop_path)
    .bind(payload.monitored.unwrap_or(true))
    .bind(&payload.library_path)
    .bind(quality_profile_id)
    .execute(&state.db)
    .await;

    let result = match result {
        Ok(v) => v,
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err((
                StatusCode::CONFLICT,
                "La película ya existe en Oberiz".into(),
            ));
        }
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    let created = find_movie(&state.db, result.last_insert_rowid())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No se pudo leer la película creada".into(),
            )
        })?;

    history::record(
        &state.db,
        "movie.added",
        &created.title,
        Some("Película añadida desde TMDB"),
        "info",
    )
    .await;

    // One-off search for whatever the tracker already has, mirroring Radarr's
    // "search on add" — RSS only ever catches releases published after this
    // point, so anything already sitting on an indexer needs this once.
    let bg = state.clone();
    let movie_id = created.id;
    tokio::spawn(async move {
        let _ = automation::run_media_cycle(&bg, "movie", movie_id).await;
    });

    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn list_movies(
    State(state): State<AppState>,
) -> Result<Json<Vec<Movie>>, (StatusCode, String)> {
    let movies = sqlx::query_as::<_, Movie>(
        r#"
        SELECT m.id, m.tmdb_id, m.title, m.original_title, m.year, m.overview,
               m.poster_path, m.backdrop_path, m.monitored, m.library_path,
               m.quality_profile_id, q.name AS quality_profile_name,
               (SELECT COUNT(*) FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1) AS file_count,
               EXISTS(SELECT 1 FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1) AS available,
               (SELECT mf.resolution FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1 ORDER BY mf.quality_score DESC,mf.id DESC LIMIT 1) AS current_resolution,
               (SELECT MAX(mf.quality_score) FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1) AS current_quality_score,
               CASE WHEN q.upgrade_allowed=1
                         AND EXISTS(SELECT 1 FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1)
                         AND COALESCE((SELECT MAX(mf.quality_score) FROM media_files mf WHERE mf.media_type='movie' AND mf.media_id=m.id AND mf.file_exists=1),0) < q.cutoff_score
                    THEN 1 ELSE 0 END AS upgrade_wanted,
               m.created_at, m.updated_at
        FROM movies m
        LEFT JOIN quality_profiles q ON q.id=m.quality_profile_id
        ORDER BY m.created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(movies))
}

pub async fn get_movie(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Movie>, (StatusCode, String)> {
    match find_movie(&state.db, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        Some(movie) => Ok(Json(movie)),
        None => Err((StatusCode::NOT_FOUND, "Película no encontrada".into())),
    }
}

pub async fn update_movie(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateMovieRequest>,
) -> Result<Json<Movie>, (StatusCode, String)> {
    if find_movie(&state.db, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .is_none()
    {
        return Err((StatusCode::NOT_FOUND, "Película no encontrada".into()));
    }

    if let Some(monitored) = payload.monitored {
        sqlx::query("UPDATE movies SET monitored = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(monitored)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    if let Some(path) = payload.library_path {
        sqlx::query(
            "UPDATE movies SET library_path = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(path)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    if let Some(profile_id) = payload.quality_profile_id {
        let profile = profiles::get_quality_profile_by_id(&state.db, profile_id).await?;
        if profile.media_type != "movie" {
            return Err((
                StatusCode::BAD_REQUEST,
                "El perfil no es de películas".into(),
            ));
        }
        sqlx::query(
            "UPDATE movies SET quality_profile_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(profile_id)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let movie = find_movie(&state.db, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Película no encontrada".into()))?;

    Ok(Json(movie))
}

pub async fn delete_movie(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let current = find_movie(&state.db, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Película no encontrada".into()))?;

    sqlx::query("DELETE FROM movies WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    history::record(
        &state.db,
        "movie.deleted",
        &current.title,
        Some("Película eliminada"),
        "info",
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}
